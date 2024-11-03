use studentgrpc::student_client::StudentClient;
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use studentgrpc::StudentRequest;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use std::thread;
use std::sync::mpsc;

pub mod studentgrpc {
    tonic::include_proto!("student_grpc");
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct StudentData {
    student: String,
    age: i32,
    faculty: String,
    discipline: i32,
}

// Estructura para manejar la respuesta del thread
#[derive(Debug)]
enum ThreadResponse {
    Success(String),
    Error(String),
}

async fn process_grpc_request(student: StudentData, host: String) -> Result<String, String> {
    let mut client = StudentClient::connect(host).await
        .map_err(|e| format!("Failed to connect to gRPC server: {}", e))?;

    let request = tonic::Request::new(StudentRequest {
        student: student.student,
        age: student.age,
        faculty: student.faculty,
        discipline: student.discipline,
    });

    client.get_student_req(request).await
        .map(|response| format!("Student: {:?}", response))
        .map_err(|e| format!("gRPC call failed: {}", e))
}

async fn handle_student(student: web::Json<StudentData>) -> impl Responder {
    // println!("Received student data: {:?}", student);

    // Seleccionamos el host de servicio según la disciplina
    let host = match student.discipline {
        1 => "http://swimming-service:50051",   // Natación
        2 => "http://athletics-service:50051",  // Atletismo
        3 => "http://boxing-service:50051",     // Boxeo
        _ => return HttpResponse::BadRequest().body("Invalid discipline"),
    };

    // Crear un canal para comunicación entre threads
    let (tx, rx) = mpsc::channel();
    let student_data = student.into_inner();
    let host_string = host.to_string();

    // Spawn un nuevo thread para manejar la solicitud gRPC
    thread::spawn(move || {
        // Crear un nuevo runtime de Tokio para el thread
        let rt = Runtime::new().unwrap();
        
        // Ejecutar la solicitud gRPC en el runtime
        let result = rt.block_on(process_grpc_request(student_data.clone(), host_string));
        
        // Enviar el resultado de vuelta al thread principal
        match result {
            Ok(response) => tx.send(ThreadResponse::Success(response)),
            Err(error) => tx.send(ThreadResponse::Error(error)),
        }.unwrap();
    });

    // Esperar la respuesta del thread
    match rx.recv().unwrap() {
        ThreadResponse::Success(response) => {
            println!("RESPONSE={}", response);
            HttpResponse::Ok().json(response)
        },
        ThreadResponse::Error(error) => {
            HttpResponse::InternalServerError().body(error)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at port 8080");
    HttpServer::new(|| {
        App::new()
            .route("/engineering", web::post().to(handle_student))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}