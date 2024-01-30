use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use std::time::Duration;
use tokio::time::sleep;




struct MemoryTracker {
    allocated_bytes: usize,
    deallocated_bytes: usize,
    allocation_count: usize,
    deallocation_count: usize,
    // Você pode adicionar mais campos conforme necessário
}

impl MemoryTracker {
    fn new() -> Self {
        MemoryTracker {
            allocated_bytes: 0,
            deallocated_bytes: 0,
            allocation_count: 0,
            deallocation_count: 0,
        }
    }

    fn allocate(&mut self, size: usize) {
        self.allocated_bytes += size;
        self.allocation_count += 1;
        // Aqui você faz a alocação real, possivelmente usando alguma função de baixo nível
    }

    fn deallocate(&mut self, size: usize) {
        self.deallocated_bytes += size;
        self.deallocation_count += 1;
        // Aqui você faz a desalocação real
    }

    // Uma função para verificar se há vazamentos de memória
    fn check_for_memory_leaks(&self) {
        if self.allocated_bytes != self.deallocated_bytes {
            println!("Possible memory leak detected!");
            // Mais detalhes e análise podem ser adicionados aqui
        }
    }

    // Funções para gerar relatórios sobre o uso de memória
    fn report(&self) {
        println!("Total allocated bytes: {}", self.allocated_bytes);
        println!("Total deallocated bytes: {}", self.deallocated_bytes);
        println!("Total allocation operations: {}", self.allocation_count);
        println!("Total deallocation operations: {}", self.deallocation_count);
        self.check_for_memory_leaks();
    }
}

static MEMORY_TRACKER: Lazy<Mutex<MemoryTracker>> = Lazy::new(|| {
    Mutex::new(MemoryTracker::new())
});




pub async  fn analyze_heap() {
    println!("Analyzing heap memory...");

    loop {
    match MEMORY_TRACKER.try_lock() {
        Ok(memory_tracker) => {
            // Agora você pode chamar métodos em MemoryTracker
            memory_tracker.report();
            break; // Sai do loop quando conseguir o bloqueio
        }
        Err(_) => {
            // Espera um pouco antes de tentar novamente
            sleep(Duration::from_millis(100)).await;
        }
    }
}

}
