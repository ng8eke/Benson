pipeline {
    agent none
    environment {
        RUST_VERSION='1.50.0'
        RUST_NIGHTLY='nightly-2021-03-05'
    }
    options{
        lock('singleton-build')
    }
    stages {
        stage('Prepare and Build with extra features') {
            agent {
                docker {
                    label 'benchmark'
                    image 'rustlang/rust:nightly'
                    args '-u root:root'
                }
            }
            steps {
                sh 'lscpu'
                sh 'free -h'
                sh '''
                  apt-get update && \
                  apt-get -y install apt-utils cmake pkg-config libssl-dev git clang libclang-dev && \
                  rustup uninstall nightly && \
                  rustup install $RUST_VERSION && \
                  rustup install $RUST_NIGHTLY && \
                  rustup default $RUST_VERSION && \
                  rustup target add --toolchain $RUST_NIGHTLY wasm32-unknown-unknown && \
                  rustup target add --toolchain $RUST_VERSION x86_64-unknown-linux-musl && \
                  mv /usr/local/rustup/toolchains/nightly* /usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu
                '''

                sh 'cargo build --release --features runtime-benchmarks'
                sh 'cargo test -p crml-staking --features runtime-benchmarks'
                sh 'cargo test -p crml-cennzx --features runtime-benchmarks'
                sh 'cargo test -p crml-nft --features runtime-benchmarks'

            }
        }

        stage('Run Benchmarks') {
            agent { label 'benchmark'}
            steps {
                sh 'rm -rf output_dir && mkdir output_dir'
                sh './target/release/benson benchmark --chain dev --steps 50 --repeat 1000 --pallet "*" --extrinsic "*" --raw --execution=wasm --wasm-execution=compiled --output output_dir'
                archiveArtifacts artifacts: 'output_dir/*'
            }
        }

        stage('Commit files back') {
            agent {
                docker {
                    label 'benchmark'
                    image 'rustlang/rust:nightly'
                    args '-u root:root'
                }
            }
            environment {
                GPG_PRIVATE_KEY = credentials('benson-bot-gpg-private-key')
                GPG_PUBLIC_KEY = credentials('benson-bot-gpg-public-key')
            }
            steps {
                sh 'mkdir clean_dir && chmod 777 clean_dir'
                dir('clean_dir'){
                    checkout([$class: 'GitSCM', branches: [[name: '${CHANGE_BRANCH}']], extensions: [], userRemoteConfigs: [[url: 'git@github.com:ng8eke/benson.git']]])
                    sh 'git checkout ${CHANGE_BRANCH}'
                    sh 'git branch'
                    sh 'cp ../output_dir/* runtime/src/weights/'
                    sh 'git config --global user.email "devops@ng8eke.io" && git config --global user.name "benson-bot"'
                    sh 'git config --global user.signingKey F7DDE7E0F924770F'
                    withCredentials([sshUserPrivateKey(credentialsId: "benson-bot-ssh-key", keyFileVariable: 'keyfile')]) {
                        sh 'mkdir -p ~/.ssh/'
                        sh 'cp ${keyfile} ~/.ssh/id_rsa'
                        sh 'ls -l ~/.ssh/'
                        sh 'ssh-keyscan -t rsa github.com >> ~/.ssh/known_hosts'
                        sh 'git status'
                        sh 'git add .'
                        sh 'git status'
                        sh 'gpg --list-keys'
                        sh 'gpg --import ${GPG_PUBLIC_KEY}'
                        sh 'gpg --allow-secret-key-import --import ${GPG_PRIVATE_KEY}'
                        sh 'gpg --list-keys'
                        sh 'git commit -S -m "Add new benchmark files `date`"; git push'
                    }
                }

            }
            post {
                always {
                    echo "clean workspace"
                    cleanWs()
                }
            }

        }
    }
}
