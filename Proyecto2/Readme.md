```sh
#Comandos
docker build -t backend-semi1 .
#Docker
docker build -t cliente_grcp .
docker tag cliente_grcp brandont2002/cliente_grcp
docker push brandont2002/cliente_grcp

docker build -t servidor_grcp .
docker tag servidor_grcp brandont2002/servidor_grcp
docker push brandont2002/servidor_grcp

docker build -t cliente_rust .
docker tag cliente_rust brandont2002/cliente_rust
docker push brandont2002/cliente_rust

docker build -t servidor_rust .
docker tag servidor_rust brandont2002/servidor_rust
docker push brandont2002/servidor_rust

docker build -t golang-consumer .
docker tag golang-consumer brandont2002/golang-consumer
docker push brandont2002/golang-consumer

docker tag brandont2002/backend-cloudrun gcr.io/so1-421118/backend-cloudrun
docker push gcr.io/so1-421118/backend-cloudrun


#GRPC
protoc --go_out=. --go-grpc_out=. cliente.proto

#Locust
locust -f traffic.py

#Gcloud
gcloud container clusters get-credentials proyecto2 --location us-centra1l-c
kubectl create namespace so1

kubectl get pods -n so1
kubectl get deployments -n so1
kubectl get services -n so1

kubectl expose deployment grpc-deployment --type=LoadBalancer --port 3000 -n so1

#grafana
kubectl port-forward -n monitoring --address 0.0.0.0 svc/grafana 3000:3000

kubectl logs deploy/consumer-deployment -n so1 -f

#REDIS
kubectl get pods -n monitoring
kubectl exec -it redis-6fbbbc7b97-pq9x9 -n monitoring -- redis-cli -a YOUR_PASSWORD

#MONGO
kubectl get pods -n mongospace
kubectl exec -it mongodb-7bcb659766-fqd7s -n mongospace -- /bin/bash
mongo -u admin -p password

mongo --host 10.100.13.216  --port 27017 -u admin -p password --authenticationDatabase admin


gcloud container clusters get-credentials proyecto2 --location us-central1-c
```