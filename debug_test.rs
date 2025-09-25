fn main() {
    let body = "<!-- specId=20250922_120000_another_spec; type=spec; v=1 -->\nContent";
    println!("Body: {:?}", body);
    
    let marker_start = body.find("<!--").unwrap();
    println!("marker_start: {}", marker_start);
    
    let tail = &body[marker_start..];
    println!("tail: {:?}", tail);
    
    let end_idx = tail.find("-->").unwrap();
    println!("end_idx: {}", end_idx);
    
    let comment = &tail[4..end_idx];
    println!("comment: {:?}", comment);
    
    for part in comment.split(';') {
        let p = part.trim();
        println!("part: {:?}", p);
        if let Some(rest) = p.strip_prefix("foundry:specId=") {
            println!("foundry:specId= match, rest: {:?}", rest);
        } else if let Some(rest) = p.strip_prefix("specId=") {
            println!("specId= match, rest: {:?}", rest);
        }
    }
}
EOF && rustc debug_test.rs && ./debug_test