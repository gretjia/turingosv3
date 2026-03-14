/// 认识论半透膜：穿透一切 <think> 与废话，只提取绝对物理基态
pub fn distill_pure_state(raw_payload: &str) -> Option<String> {
    let start_tag = "[State:";
    
    // 高级审美：使用 rfind 从后往前搜索。
    // 因为模型在冗长的 <think> 中可能会多次推演 [State:...]，
    // 我们只提取它思考完毕后的【最终物理决定】！
    if let Some(start_idx) = raw_payload.rfind(start_tag) {
        let slice = &raw_payload[start_idx..];
        if let Some(end_offset) = slice.find(']') {
            // 重新锻造为绝对干净的内核态 ABI 格式
            return Some(slice[..=end_offset].trim().to_string());
        }
    }
    
    // 如果没有找到严谨的格式边界，截获幽灵载荷，直接判定该分叉宇宙死亡
    None
}
