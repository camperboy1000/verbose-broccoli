FROM rust:bullseye

WORKDIR /usr/src/laundry-api
COPY . .

RUN cargo install --path .

CMD [ "laundry-api" ]
