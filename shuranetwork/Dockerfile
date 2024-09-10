FROM rust:latest

WORKDIR /app
COPY target/debug/edblock /app/

EXPOSE 5000
EXPOSE 1001

CMD ["./edblock"]