//! Gerencia o estado do chat, como usuários conectados e histórico de mensagens.
//!
//! Isso suporta a fusão de vários estados juntos, visando facilitar a consistência.
//! Note que isso é feito sem o auxílio de carimbos de data/hora.

use crate::history::History;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Representa o tipo de mensagem sendo enviada, ditando
#[derive(Serialize, Deserialize, Debug, Clone)]


// ... [resto da estrutura Message]



pub enum MessageType {
    /// Uma mensagem enviada por um usuário, contendo texto que deve ser exibido
    /// para o destinatário.
    Message,
    /// Uma mensagem que representa o estado.
    State,
}

/// Uma mensagem ou evento gerado por um usuário, pode conter vários dados.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    /// O tipo de mensagem.
    pub message_type: MessageType,
    /// Os dados contidos na mensagem, representados como um vetor de bytes.
    pub data: Vec<u8>, // Seria melhor usar um valor emprestado aqui, já que os vetores alocam na heap
    /// O destinatário pretendido da mensagem, um PeerId codificado como uma string.
    pub addressee: Option<String>,
    /// O remetente desta mensagem, PeerId codificado como uma String
    pub source: String,
}

/// Contém o estado atual da aplicação de chat.
#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    /// O histórico de mensagens do chat.
    pub history: History<Message>,
    /// Os nomes de usuário de todos atualmente conectados à rede.
    /// Isso é um mapeamento de `PeerId` para `String`. Os `PeerId` são codificados
    /// como Strings.
    pub usernames: HashMap<String, String>,
}

impl State {
    /// Tenta mesclar dois estados.
    /// Note que se nosso histórico estiver atualizado, então não aceitaremos esse novo histórico.
    pub fn merge(&mut self, mut other: State) {
        // Mescla sempre os nomes de usuário!
        for (peer_id, username) in other.usernames.drain() {
            if !self.usernames.contains_key(&peer_id) {
                println!("{} entrou no chat!", &username);
                self.usernames.insert(peer_id, username);
            }
        }

        // Mescla Mensagens
        // Observe que só queremos mesclar mensagens no caso de não termos histórico
        // Isso impede que mensagens sejam abusadas por meio de spam!
        if self.history.get_count() < 1 && other.history.get_count() > 1 {
            // Inicia a mesclagem
            for message in other.history.get_all() {
                println!(
                    "{}: {}",
                    self.get_username(&message.source),
                    String::from_utf8_lossy(&message.data)
                );
                self.history.insert((*message).to_owned());
            }
        }
    }

    /// Tenta obter o nome de usuário de um usuário; se o usuário não existir, o nome de usuário
    /// padrão é `anon`.
    pub fn get_username(&self, usr: &String) -> String {
        self.usernames
            .get(usr)
            .unwrap_or(&String::from("anonimo"))
            .to_string()
    }
}



