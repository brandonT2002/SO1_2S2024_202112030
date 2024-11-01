from locust import HttpUser, task, between
import random

class FacultyUser(HttpUser):
    wait_time = between(1, 5)

    @task
    def send_request(self):
        student = {
            "student": f"Student {random.randint(1, 100)}",
            "age": random.randint(18, 30),
            "faculty": random.choice(["Ingeniería", "Agronomía"]),
            "discipline": random.choice([1, 2, 3])
        }
        self.client.post("/api/students", json=student)
