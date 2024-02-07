FROM    mcr.microsoft.com/devcontainers/rust:1-1-bullseye

RUN     sudo apt update && sudo apt upgrade -y && sudo apt-get install -y iputils-ping && \
        rm -rf /etc/localtime && ln -s /usr/share/zoneinfo/${TZ} /etc/localtime

USER    vscode

