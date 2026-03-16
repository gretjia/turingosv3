sed -i 's/last_state = pure_state;/last_state = format!("{}\\n  {}", last_state, pure_state);/g' src/swarm.rs
