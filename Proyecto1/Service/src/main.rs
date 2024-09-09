use std::{fs::File, io::{self, Read}, path::{Path, PathBuf}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SystemInfo {
    #[serde(rename = "Processes")]
    processes: Vec<Process>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Process {
    #[serde(rename = "PID")]
    pid: u32,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Cmdline")]
    cmd_line: String,
    #[serde(rename = "MemoryUsage")]
    memory_usage: f64,
    #[serde(rename = "CPUUsage")]
    cpuy_usage: f64,
}

impl Process {
    fn get_container_id(&self) -> String {
        let parts: Vec<&str> = self.cmd_line.split_whitespace().collect();
        for(i, part) in parts.iter().enumerate() {
            if *part == "-id" {
                if let Some(id) = parts.get(i + 1) {
                    return id.to_string();
                }
            }
        }
        
    }
}

fn read_proc_file(file_name: &str) -> Result<String, io::Error> {
    let path: PathBuf = Path::new("/proc").join(file_name);
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(content)
}

fn parse_proc_to_struct(json_str: &str) -> Result<SystemInfo, serde_json::Error> {
    let system_info: SystemInfo = serde_json::from_str(json_str)?;
    Ok(system_info)
}

fn main() {
    let json_str = read_proc_file("sysinfo_202112030").unwrap();
    let system_info = parse_proc_to_struct(&json_str);

    match system_info {
        Ok(info) => {
            println!("System Info: {:?}", info);
        },
        Err(e) => {
            println!("Error en el parser: {:?}", e);
        }
    }
}
