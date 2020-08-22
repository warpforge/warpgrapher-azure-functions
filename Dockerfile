# syntax=docker/dockerfile-upstream:experimental

FROM rust:slim AS build-image

ENV PATH "$PATH:/root/.cargo/bin"
ENV DOTNET_SKIP_FIRST_TIME_EXPERIENCE true
ENV DOTNET_CLI_TELEMETRY_OPTOUT true

RUN apt-get update \
    && apt-get upgrade -y \
    && apt-get install -y wget unzip apt-transport-https gnupg \
    && wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > microsoft.asc.gpg \
    && mv microsoft.asc.gpg /etc/apt/trusted.gpg.d/ \
    && wget -q https://packages.microsoft.com/config/debian/9/prod.list \
    && mv prod.list /etc/apt/sources.list.d/microsoft-prod.list \
    && apt-get update \
    && wget https://github.com/google/protobuf/releases/download/v3.6.1/protoc-3.6.1-linux-x86_64.zip \
    && unzip protoc-3.6.1-linux-x86_64.zip -d /usr \
    && rm protoc-3.6.1-linux-x86_64.zip \
    && apt-get install -y dotnet-sdk-2.2 \
    && apt-get install -y openssl libssl-dev \
    && apt-get install -y pkg-config \
    && apt-get remove -y --purge wget unzip apt-transport-https gnupg \
    && apt-get autoremove -y \
    && apt-get clean \
    && rm -rf /usr/share/dotnet/sdk/NuGetFallbackFolder/*

WORKDIR /src
COPY . /src

# Run with mounted cache
RUN --mount=type=cache,target=/src/target \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    ["cargo", "run", "--release", "--", "init", "--script-root", "/home/site/wwwroot", "--sync-extensions"]

FROM mcr.microsoft.com/azure-functions/base:2.0 as runtime-image

FROM mcr.microsoft.com/dotnet/core/runtime-deps:2.2

ENV AzureWebJobsScriptRoot=/home/site/wwwroot \
    HOME=/home \
    FUNCTIONS_WORKER_RUNTIME=Rust \
    languageWorkers__workersDirectory=/home/site/wwwroot/workers

# Copy the Azure Functions host from the runtime image
COPY --from=runtime-image [ "/azure-functions-host", "/azure-functions-host" ]

# Copy the script root contents from the build image
COPY --from=build-image ["/home/site/wwwroot", "/home/site/wwwroot"]

WORKDIR /home/site/wwwroot
CMD [ "/azure-functions-host/Microsoft.Azure.WebJobs.Script.WebHost" ]
