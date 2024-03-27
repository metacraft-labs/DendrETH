FROM ubuntu:22.04

RUN echo 'deb [trusted=yes]  http://deb.nil.foundation/ubuntu/ all main' >>/etc/apt/sources.list
RUN apt update && apt -y upgrade
RUN apt install -y zkllvm proof-producer
RUN apt install -y vim libyaml-cpp-dev cmake clang-format libboost-all-dev
RUN apt update && apt -y upgrade

ENTRYPOINT [ "/bin/bash", "-l", "-c" ]