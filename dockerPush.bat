@echo off
echo Building and pushing HFT Simulator images to Docker Hub...

docker build -t 1dothings/hft-exchange:latest .
docker build -t 1dothings/hft-frontend:latest ./frontend

docker push 1dothings/hft-exchange:latest
docker push 1dothings/hft-frontend:latest

echo Done! Images pushed to Docker Hub.