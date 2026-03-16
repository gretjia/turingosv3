cat << 'INNER_EOF' > patch_file.diff
--- src/bin/batch_evaluator.rs
+++ src/bin/batch_evaluator.rs
@@ -107,6 +107,9 @@
         
         let content = fs::read_to_string(path).expect("Unable to read lean file");
         
+        let rt = tokio::runtime::Runtime::new().unwrap();
+        let _guard = rt.enter();
+        
         let sentinel = minif2f_swarm::wal::WalSentinel::new(format!("/tmp/{}.wal", file_name));
         let agent = SpeculativeSwarmAgent::new(&api_url, &model_name, max_steps_per_theorem, swarm_size, timeout_secs, sentinel, vec![]);
         
INNER_EOF
patch src/bin/batch_evaluator.rs < patch_file.diff
