// Importação dos módulos necessários
mod history;
mod state;

// Importação de bibliotecas externas
use futures::StreamExt;
use history::History;
use libp2p::{
    core::upgrade,
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity::{self},
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::NetworkBehaviourEventProcess,
    tcp::TcpConfig,
    NetworkBehaviour, PeerId, Swarm, Transport,
};
use log::{error, info};
use state::{Message, MessageType, State};
use std::{collections::HashMap, process};
use tokio::{io::AsyncBufReadExt, sync::mpsc, signal::ctrl_c};



// Função para transmitir uma resposta para outros pares utilizando canais
fn send_response(message: Message, sender: mpsc::UnboundedSender<Message>) {
    tokio::spawn(async move {
        if let Err(e) = sender.send(message) {
            error!("error sending response via channel {}", e);
        }
    });
}

// Função para enviar uma mensagem usando o swarm
fn send_message(message: &Message, swarm: &mut Swarm<Chat>, topic: &Topic) {
    let bytes = bincode::serialize(message).unwrap();
    swarm
        .behaviour_mut()
        .messager
        .publish(topic.clone(), bytes);
}


// Definição da estrutura de rede (NetworkBehaviour)
#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
struct Chat {
    dns: Mdns,
    messager: Floodsub,
    #[behaviour(ignore)]
    state: State,
    #[behaviour(ignore)]
    peer_id: String,
    #[behaviour(ignore)]
    responder: mpsc::UnboundedSender<Message>,
}

// Implementação do processamento de eventos Mdns na estrutura de rede
impl NetworkBehaviourEventProcess<MdnsEvent> for Chat {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(nodes) => {
                // Novo nó encontrado!
                for (peer, addr) in nodes {
                    info!("Peer {} found at {}", peer, addr);
                    self.messager.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(nodes) => {
                // Manipular expiração de nós
                for (peer, addr) in nodes {
                    if !self.dns.has_node(&peer) {
                        info!("Peer {} disconnected at {}", peer, addr);
                        self.messager.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}


// Implementação do processamento de eventos Floodsub na estrutura de rede
impl NetworkBehaviourEventProcess<FloodsubEvent> for Chat {
    fn inject_event(&mut self, event: FloodsubEvent) {
        match event {
            FloodsubEvent::Message(raw_data) => {
                // Analisar a mensagem como bytes
                let deser = bincode::deserialize::<Message>(&raw_data.data);
                if let Ok(message) = deser {
                    if let Some(user) = &message.addressee {
                        if *user != self.peer_id.to_string() {
                            return; // Não processar mensagens não destinadas a nós.
                        }
                    }

                    match message.message_type {
                        MessageType::Message => {
                            let username: String =
                                self.state.get_username(&raw_data.source.to_string());
                            println!("{}: {}", username, String::from_utf8_lossy(&message.data));

                            // Armazenar mensagem no histórico
                            self.state.history.insert(message);
                        }
                        MessageType::State => {
                            info!("History received!");
                            let data: State = bincode::deserialize(&message.data).unwrap();
                            self.state.merge(data);
                        }
                    }
                } else {
                    error!("Unable to decode message! Due to {:?}", deser.unwrap_err());
                }
            }
            FloodsubEvent::Subscribed { peer_id, topic: _ } => {
                // Enviar nosso estado para o novo usuário
                info!("Sending stage to {}", peer_id);
                let message: Message = Message {
                    message_type: MessageType::State,
                    data: bincode::serialize(&self.state).unwrap(),
                    addressee: Some(peer_id.to_string()),
                    source: self.peer_id.to_string(),
                };
                send_response(message, self.responder.clone());
            }
            FloodsubEvent::Unsubscribed { peer_id, topic: _ } => {
                let name = self
                    .state
                    .usernames
                    .remove(&peer_id.to_string())
                    .unwrap_or(String::from("Anon"));
                println!("{} has left the chat.", name);
            }
        }
    }
}


// Função principal assíncrona
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    // Geração de chaves e identificação do peer local
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("Local peer ID: {}", peer_id);

    // Configuração das chaves de autenticação
    let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&id_keys)
        .expect("unable to create authenticated keys");

    // Configuração do transporte
    let transport = TcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    // Geração de canal para respostas
    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();

    // Configuração da leitura da entrada padrão
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    // Solicita ao usuário que insira um nome de usuário
    print!("Please enter a username: \n");
    let username = stdin
        .next_line()
        .await
        .expect("a valid username")
        .unwrap_or(String::from("anon"))
        .trim()
        .to_owned();

    // Inicialização da estrutura de rede
    let mut behaviour = Chat {
        dns: Mdns::new(MdnsConfig::default())
            .await
            .expect("unable to create mdns"),
        messager: Floodsub::new(peer_id.clone()),
        state: State {
            history: History::new(),
            usernames: HashMap::from([(peer_id.to_string(), username)]),
        },
        peer_id: peer_id.to_string(),
        responder: response_sender,
    };

    // Definição do tópico do chat
    let topic = Topic::new("sylo");
    behaviour.messager.subscribe(topic.clone());

    // Inicialização do swarm
    let mut swarm = Swarm::new(transport, behaviour, peer_id.clone());
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
        tokio::select! {
            line = stdin.next_line() => {



                if let Some(input_line) = line.expect("a valid line") {
                    let trimmed_line = input_line.trim();

                    if trimmed_line.starts_with("/msg") {
                        // Verifica se há conteúdo após "/msg "
                        if trimmed_line.len() > 5 {
                            let content = &trimmed_line[5..];
                            let parts: Vec<&str> = content.splitn(2, ' ').collect();
                            if parts.len() == 2 {
                                let addressee = parts[0].trim();
                                let message_content = parts[1].trim();
            
                                let message: Message = Message {
                                    message_type: MessageType::Message,
                                    data: message_content.as_bytes().to_vec(),
                                    addressee: Some(addressee.to_string()),
                                    source: peer_id.to_string(),
                                };
            
                                send_message(&message, &mut swarm, &topic);
            
                                // Registrar mensagem no histórico
                                swarm.behaviour_mut().state.history.insert(message);
                                continue; // Continua para a próxima iteração do loop
                            } else {
                                println!("Uso: /msg <PeerId> <mensagem>");
                            }
                        } else {
                            println!("Uso: /msg <PeerId> <mensagem>");
                        }
                    }
    
                    // Trata o comando /kick
                    if trimmed_line.starts_with("/kick ") {
                        let peer_id_str = trimmed_line[6..].trim();
                        kick_user(peer_id_str, &mut swarm.behaviour_mut().state);
                        continue; // Continua para a próxima iteração do loop, ignorando o processamento adicional
                    }
    
                    // Trata o comando /users
                    if trimmed_line == "/users" {
                        list_active_users(&swarm.behaviour().state);
                        continue; // Continua para a próxima iteração do loop, ignorando o processamento adicional
                    }
    
                    // Processa a mensagem normal
                    let message: Message = Message {
                        message_type: MessageType::Message,
                        data: input_line.as_bytes().to_vec(),
                        addressee: None,
                        source: peer_id.to_string(),
                    };
    
                    send_message(&message, &mut swarm, &topic);
    
                    // Registrar mensagem no histórico
                    swarm.behaviour_mut().state.history.insert(message);
                }
            },
            event = swarm.select_next_some() => {
                info!("Swarm event: {:?}", event);
            },
            response = response_rcv.recv() => {
                if let Some(message) = response {
                    send_message(&message, &mut swarm, &topic);
                }
            },
            event = ctrl_c() => {
                if let Err(e) = event {
                    println!("Failed to register interrupt handler {}", e);
                }
                break;
            }

        }
    }
    
    fn list_active_users(state: &State) {
        println!("Usuários ativos:");
        for (peer_id, username) in state.usernames.iter() {
            println!("- {}: {}", peer_id, username);
        }
    }

    fn kick_user(peer_id_str: &str, state: &mut State) {    
        if state.usernames.remove(peer_id_str).is_some() {
            println!("User, peer_id: {} removeted.", peer_id_str);
        } else {
            println!("Usuário {} não foi encontrado.", peer_id_str);
        }
    }


    
    
    
    
    
    

    // Desinscrever-se do tópico
    swarm.behaviour_mut().messager.unsubscribe(topic);

    // HACK: Solução alternativa para forçar a desinscrição a ser realmente enviada.
    // Chato, pois causa um atraso ao fechar a aplicação.
    swarm.select_next_some().await;

    process::exit(0);
}
