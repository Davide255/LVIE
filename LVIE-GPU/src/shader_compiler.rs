use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

pub fn build(path: &str) -> String {
    let FUNCTIONS: HashMap<&str, &str> = HashMap::from([
        ("CLAMP", 
r#"fn clamp(start: f32, stop: f32, value: f32) -> f32 {
    if (value > stop) {return stop;}
    else if (value < start) {return start;}
    else {return value;}
}"#)
    ]);

    let mut file = File::open(path).expect("Failed to open file");
    
    let mut shader = String::new();
    file.read_to_string(&mut shader).expect("Failed to read file");

    let rt = shader.clone();

    if shader.starts_with("import") {
        shader = shader[shader.find('}').unwrap() + 1..].to_string();

        let import_block_args = rt[rt.find('{').unwrap() + 1..rt.find('}').unwrap()].trim().split(' ');

        for arg in import_block_args.clone() {
            if FUNCTIONS.contains_key(arg.to_uppercase().as_str()) {
                shader = String::from(*FUNCTIONS.get(arg.to_uppercase().as_str()).unwrap()) + &shader;
            }
        }
    }

    shader
}

#[cfg(test)]
mod test {

}
