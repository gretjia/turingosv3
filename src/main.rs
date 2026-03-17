use turingosv3::kernel::{Kernel, File};
use turingosv3::bus::{TuringBus, ThermodynamicHeartbeatTool, MembraneGuardTool, WalSnapshotTool};

fn main() {
    println!(">>> TuringOS v3 Microkernel Booting... <<<");

    let kernel = Kernel::new("step_20".to_string());
    let mut bus = TuringBus::new(kernel);

    // Mount skills
    bus.mount_tool(Box::new(ThermodynamicHeartbeatTool::new(10)));
    bus.mount_tool(Box::new(MembraneGuardTool));
    bus.mount_tool(Box::new(WalSnapshotTool));

    println!(">>> Skills mounted. Starting swarm... <<<");

    // Simulate appending files
    for i in 1..=25 {
        let file = File {
            id: format!("file_{}", i),
            author: "agent_alpha".to_string(),
            payload: format!("knowledge payload #{}", i),
            citations: vec![],
            stake: 100,
            intrinsic_reward: 0.0,
            price: 0.0,
        };

        match bus.append(file) {
            Ok(_) => {
                println!("[Tick {}] [+] ACCEPTED. File Appended.", i);
                bus.tick_map_reduce();
            }
            Err(e) => {
                println!("[Tick {}] [-] REJECTED. Reason: {}", i, e);
            }
        }
    }

    println!(">>> TuringOS Simulation Completed. <<<");
}
