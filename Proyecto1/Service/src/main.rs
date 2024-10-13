use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use chrono;
use tokio::signal;
use reqwest;

// CREACIÓN DE STRUCT
#[derive(Debug, Serialize, Deserialize)]
struct SystemInfo {
    #[serde(rename = "Processes")]
    processes: Vec<Process>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
    cpu_usage: f64,
}

#[derive(Debug, Serialize, Clone)]
struct LogProcess {
    pid: u32,
    container_id: String,
    name: String,
    memory_usage: f64,
    cpu_usage: f64,
}

#[derive(Debug, Serialize)]
struct LogData {
    pid: u32,
    container_id: String,
    name: String,
    vsz: u32,
    rss: u32,
    memory_usage: f64,
    cpu_usage: f64,
    action: String,
    timestamp: String,
}

// IMPLEMENTACIÓN DE MÉTODOS
impl Process {
    fn get_container_id(&self) -> &str {
        let parts: Vec<&str> = self.cmd_line.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "-id" {
                if let Some(id) = parts.get(i + 1) {
                    return id;
                }
            }
        }
        "N/A"
    }
}

// IMPLEMENTACIÓN DE TRAITS
impl Eq for Process {}

impl Ord for Process {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cpu_usage.partial_cmp(&other.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| self.memory_usage.partial_cmp(&other.memory_usage).unwrap_or(std::cmp::Ordering::Equal))
    }
}

impl PartialOrd for Process {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// FUNCIONES
// Función para enviar los datos a la API
async fn send_logs_to_api(logs: &[LogData]) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client.post("http://127.0.0.1:8000/logs")
        .json(&logs)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Logs sent successfully.");
    } else {
        eprintln!("Failed to send logs. Status: {}", response.status());
    }

    Ok(())
}

async fn send_close_request() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client.post("http://127.0.0.1:8000/close")
        .send()
        .await?;

    if response.status().is_success() {
        println!("Close request sent successfully.");
    } else {
        eprintln!("Failed to send close request. Status: {}", response.status());
        let body = response.text().await?;
        eprintln!("Response body: {}", body);
    }

    Ok(())
}

fn get_docker_container_id() -> String {
    let output = std::process::Command::new("sudo")
        .arg("docker")
        .arg("compose")
        .arg("--file")
        .arg("/home/jefferson/Escritorio/lab-sopes1/Proyecto1/API/docker-compose.yaml")
        .arg("ps")
        .arg("-q")
        .output()
        .expect("Failed to execute docker compose ps -q");

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if container_id.is_empty() {
        "N/A".to_string()
    } else {
        container_id
    }
}

fn kill_container(id: &str) -> std::process::Output {
    let output = std::process::Command::new("sudo")
        .arg("docker")
        .arg("stop")
        .arg(id)
        .output()
        .expect("Failed to execute process");

    println!("Killing container with id: {}", id);
    output
}

fn remove_cronjob() {
    // Comando para eliminar el cronjob específico
    let cronjob_command = r"
    crontab -l | grep -v '/home/jefferson/Escritorio/lab-sopes1/Proyecto1/Docker/Cronjob.sh' | crontab -
    ";
    
    let output = Command::new("bash")
        .arg("-c")
        .arg(cronjob_command)
        .output()
        .expect("Failed to execute cronjob removal command");

    if !output.status.success() {
        eprintln!("Failed to remove cronjob: {}", String::from_utf8_lossy(&output.stderr));
    }
}

async fn analyzer(system_info: SystemInfo) {
    let mut log_proc_list: Vec<LogProcess> = Vec::new();
    let mut processes_list: Vec<Process> = system_info.processes;

    processes_list.sort();
    let (lowest_list, highest_list) = processes_list.split_at(processes_list.len() / 2);

    let container_id = get_docker_container_id();

    // Datos para las columnas
    let headers = ["PID", "Name", "Container ID", "Memory Usage", "CPU Usage"];

    // Función para calcular el ancho máximo de cada columna
    fn calculate_column_widths(processes: &[Process], headers: &[&str]) -> Vec<usize> {
        let mut widths = headers.iter().map(|h| h.len()).collect::<Vec<_>>();
        
        for process in processes {
            widths[0] = widths[0].max(process.pid.to_string().len()); // PID
            widths[1] = widths[1].max(process.name.len());             // Name
            widths[2] = widths[2].max(process.get_container_id().len()); // Container ID
            widths[3] = widths[3].max(process.memory_usage.to_string().len()); // Memory Usage
            widths[4] = widths[4].max(process.cpu_usage.to_string().len());    // CPU Usage
        }
        widths
    }

    // Calcular el ancho de las columnas basado en el contenido de procesos
    let column_widths = calculate_column_widths(&processes_list, &headers);

    // Calcular el ancho total de la tabla
    let total_width = column_widths.iter().sum::<usize>() + column_widths.len() * 3 + 1; // +3 por los separadores y padding

    // Función para generar una línea de separación para las tablas
    fn print_separator(column_widths: &[usize], left: char, mid: char, right: char) {
        print!("{}", left);
        for (i, width) in column_widths.iter().enumerate() {
            print!("{}", "═".repeat(*width + 2)); // +2 para el padding
            if i < column_widths.len() - 1 {
                print!("{}", mid);
            }
        }
        println!("{}", right);
    }

    // Función para imprimir los encabezados o filas de la tabla
    fn print_row(values: &[&str], column_widths: &[usize]) {
        print!("║");
        for (value, width) in values.iter().zip(column_widths.iter()) {
            print!(" {:^width$} ║", value, width = *width);
        }
        println!();
    }

    // Sección Bajo Consumo
    println!("╔{:═^width$}╗", " Bajo Consumo ", width = total_width - 2); // -2 para los bordes
    print_separator(&column_widths, '╠', '╦', '╣');

    print_row(&headers, &column_widths);
    print_separator(&column_widths, '╠', '╬', '╣');

    for process in lowest_list {
        print_row(&[
            &process.pid.to_string(),
            &process.name,
            &process.get_container_id(),
            &process.memory_usage.to_string(),
            &process.cpu_usage.to_string(),
        ], &column_widths);
    }
    print_separator(&column_widths, '╚', '╩', '╝');

    // Sección Alto Consumo
    println!();
    
    println!("╔{:═^width$}╗", " Alto Consumo ", width = total_width - 2); // -2 para los bordes
    print_separator(&column_widths, '╠', '╦', '╣');

    print_row(&headers, &column_widths);
    print_separator(&column_widths, '╠', '╬', '╣');

    for process in highest_list {
        print_row(&[
            &process.pid.to_string(),
            &process.name,
            &process.get_container_id(),
            &process.memory_usage.to_string(),
            &process.cpu_usage.to_string(),
        ], &column_widths);
    }
    print_separator(&column_widths, '╚', '╩', '╝');

    // Proceso de matado de contenedores
    if lowest_list.len() > 3 {
        for process in lowest_list.iter().skip(3) {
            if container_id != process.get_container_id() {
                let log_process = LogProcess {
                    pid: process.pid,
                    container_id: process.get_container_id().to_string(),
                    name: process.name.clone(),
                    memory_usage: process.memory_usage,
                    cpu_usage: process.cpu_usage,
                };
    
                log_proc_list.push(log_process.clone());
    
                // Matamos el contenedor.
                let _output = kill_container(&process.get_container_id());
            }
        }
    }

    if highest_list.len() > 2 {
        for process in highest_list.iter().take(highest_list.len() - 2) {
            if container_id != process.get_container_id() {
                let log_process = LogProcess {
                    pid: process.pid,
                    container_id: process.get_container_id().to_string(),
                    name: process.name.clone(),
                    memory_usage: process.memory_usage,
                    cpu_usage: process.cpu_usage
                };
                log_proc_list.push(log_process.clone());
                let _output = kill_container(&process.get_container_id());
            }
        }
    }

    // Convertir LogProcess a LogData
    let log_data_list: Vec<LogData> = log_proc_list.iter().map(|log| LogData {
        pid: log.pid,
        container_id: log.container_id.clone(),
        name: log.name.clone(),
        vsz: 0,
        rss: 0,
        memory_usage: log.memory_usage,
        cpu_usage: log.cpu_usage,
        action: "stopped".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }).collect();

    // Enviar los datos a la API
    if let Err(e) = send_logs_to_api(&log_data_list).await {
        eprintln!("Error sending logs: {}", e);
    }
}

fn read_proc_file(file_name: &str) -> io::Result<String> {
    let path  = Path::new("/proc").join(file_name);
    let mut file = File::open(path)?;
    let mut content = String::new();

    file.read_to_string(&mut content)?;
    Ok(content)
}

fn parse_proc_to_struct(json_str: &str) -> Result<SystemInfo, serde_json::Error> {
    let system_info: SystemInfo = serde_json::from_str(json_str)?;
    Ok(system_info)
}

#[tokio::main]
async fn main() {
    let exit = Arc::new(Mutex::new(false));
    let exit_clone = Arc::clone(&exit);

    // Crear una tarea para manejar la señal SIGINT
    let _signal_handler = tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to setup signal handler");
        let mut exit = exit_clone.lock().unwrap();
        *exit = true;
    });

    while !*exit.lock().unwrap() {
        let system_info: Result<SystemInfo, _>;
        let json_str = read_proc_file("sysinfo_202112030").unwrap();
        system_info = parse_proc_to_struct(&json_str);

        match system_info {
            Ok(info) => {
                analyzer(info).await;
            }
            Err(e) => println!("Failed to parse JSON: {}", e),
        }

        std::thread::sleep(std::time::Duration::from_secs(10));
    }

    // Enviar petición final al endpoint /close
    if let Err(e) = send_close_request().await {
        eprintln!("Error sending close request: {}", e);
    }

    // Eliminar cronjob
    remove_cronjob();
}
