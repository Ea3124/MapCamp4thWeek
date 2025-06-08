

client 

docker build -f Dockerfile.client -t client-builder .
docker run --rm -v "$(pwd)/output:/output" client-builder \
    cp /app/client/target/release/client /output/


server

./run.sha2


exe와 app 배포