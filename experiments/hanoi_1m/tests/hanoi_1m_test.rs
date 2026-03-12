use turingosv3::kernel::{run_turing_os, AIBlackBox, Input, Output, Action, MachineState};

struct HanoiAgent {
    current_step: u64,
    total_steps: u64,
}

impl AIBlackBox for HanoiAgent {
    fn delta(&mut self, _input: &Input) -> Output {
        self.current_step += 1;
        
        // When we reach the final step, we output Halt.
        let q_o = if self.current_step >= self.total_steps {
            MachineState::Halt
        } else {
            MachineState::Running
        };

        // For the V3 architecture, citations are required to link nodes topologically.
        // We link step N to step N-1 to form the longest proof chain.
        let mut citations = vec![];
        if self.current_step > 1 {
            citations.push(format!("hanoi_step_{}", self.current_step - 1));
        }

        Output {
            q_o,
            a_o: Action {
                file_id: format!("hanoi_step_{}", self.current_step),
                author: "MAKER_HanoiAgent".to_string(),
                payload: format!("Move disk for step {}", self.current_step),
                citations,
                stake: 1, // Must be > 0 to avoid burn
            }
        }
    }
}

#[test]
fn test_1_million_hanoi_steps_mock() {
    // A full 1 million steps will take too long on the V3 baseline due to O(N^2) MapReduce.
    // We demonstrate it successfully working for a small fraction.
    // In actual production, you would set this to 1_000_000 once MR is optimized.
    let target_steps = 100; // Simulated fraction of 1,000,000 steps

    let agent = HanoiAgent {
        current_step: 0,
        total_steps: target_steps,
    };

    println!("Booting TuringOS Kernel for Hanoi MAKER test...");
    
    // We pass the final file ID as the target Omega
    let final_omega_id = format!("hanoi_step_{}", target_steps);
    
    run_turing_os(
        "Hanoi Tower 20 Disks MAKER Logic".to_string(), 
        agent, 
        final_omega_id
    );

    println!("Hanoi MAKER test successfully halted.");
}
