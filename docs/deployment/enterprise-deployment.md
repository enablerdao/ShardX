# エンタープライズデプロイガイド

このガイドでは、ShardXをエンタープライズ環境にデプロイするための詳細な手順と推奨事項を説明します。

## 目次

- [AWS](#aws)
- [Google Cloud Platform](#google-cloud-platform)
- [Microsoft Azure](#microsoft-azure)
- [オンプレミス環境](#オンプレミス環境)
- [Kubernetes](#kubernetes)
- [高可用性構成](#高可用性構成)
- [セキュリティ推奨事項](#セキュリティ推奨事項)
- [パフォーマンスチューニング](#パフォーマンスチューニング)

## AWS

Amazon Web Services（AWS）は、幅広いクラウドサービスを提供するプラットフォームです。ShardXをAWSにデプロイする方法はいくつかあります。

### ECS (Elastic Container Service)

ECSは、コンテナ化されたアプリケーションを簡単に実行、停止、管理できるコンテナオーケストレーションサービスです。

#### デプロイ手順

1. **前提条件**:
   - AWSアカウント
   - AWS CLI
   - Docker

2. **Dockerイメージの作成とプッシュ**:
   ```bash
   # ECRリポジトリを作成
   aws ecr create-repository --repository-name shardx

   # ECRにログイン
   aws ecr get-login-password | docker login --username AWS --password-stdin <your-account-id>.dkr.ecr.<region>.amazonaws.com

   # イメージをビルドしてタグ付け
   docker build -t <your-account-id>.dkr.ecr.<region>.amazonaws.com/shardx:latest .

   # イメージをプッシュ
   docker push <your-account-id>.dkr.ecr.<region>.amazonaws.com/shardx:latest
   ```

3. **ECSクラスターの作成**:
   ```bash
   aws ecs create-cluster --cluster-name shardx-cluster
   ```

4. **タスク定義の作成**:
   ```bash
   # task-definition.jsonファイルを作成
   cat > task-definition.json << EOF
   {
     "family": "shardx",
     "networkMode": "awsvpc",
     "executionRoleArn": "arn:aws:iam::<your-account-id>:role/ecsTaskExecutionRole",
     "containerDefinitions": [
       {
         "name": "shardx",
         "image": "<your-account-id>.dkr.ecr.<region>.amazonaws.com/shardx:latest",
         "essential": true,
         "portMappings": [
           {
             "containerPort": 54868,
             "hostPort": 54868,
             "protocol": "tcp"
           },
           {
             "containerPort": 54867,
             "hostPort": 54867,
             "protocol": "tcp"
           }
         ],
         "environment": [
           {
             "name": "RUST_LOG",
             "value": "info"
           },
           {
             "name": "INITIAL_SHARDS",
             "value": "64"
           }
         ],
         "logConfiguration": {
           "logDriver": "awslogs",
           "options": {
             "awslogs-group": "/ecs/shardx",
             "awslogs-region": "<region>",
             "awslogs-stream-prefix": "ecs"
           }
         }
       }
     ],
     "requiresCompatibilities": [
       "FARGATE"
     ],
     "cpu": "1024",
     "memory": "2048"
   }
   EOF

   # タスク定義を登録
   aws ecs register-task-definition --cli-input-json file://task-definition.json
   ```

5. **サービスの作成**:
   ```bash
   aws ecs create-service \
     --cluster shardx-cluster \
     --service-name shardx-service \
     --task-definition shardx:1 \
     --desired-count 2 \
     --launch-type FARGATE \
     --network-configuration "awsvpcConfiguration={subnets=[subnet-12345678,subnet-87654321],securityGroups=[sg-12345678],assignPublicIp=ENABLED}"
   ```

### EKS (Elastic Kubernetes Service)

EKSは、AWSでKubernetesを実行するためのマネージドサービスです。

#### デプロイ手順

1. **EKSクラスターの作成**:
   ```bash
   eksctl create cluster \
     --name shardx-cluster \
     --version 1.24 \
     --region <region> \
     --nodegroup-name standard-nodes \
     --node-type t3.medium \
     --nodes 3 \
     --nodes-min 1 \
     --nodes-max 5 \
     --managed
   ```

2. **Kubernetesマニフェストの作成**:
   ```bash
   # deployment.yamlファイルを作成
   cat > deployment.yaml << EOF
   apiVersion: apps/v1
   kind: Deployment
   metadata:
     name: shardx
     labels:
       app: shardx
   spec:
     replicas: 3
     selector:
       matchLabels:
         app: shardx
     template:
       metadata:
         labels:
           app: shardx
       spec:
         containers:
         - name: shardx
           image: <your-account-id>.dkr.ecr.<region>.amazonaws.com/shardx:latest
           ports:
           - containerPort: 54868
           - containerPort: 54867
           env:
           - name: RUST_LOG
             value: "info"
           - name: INITIAL_SHARDS
             value: "64"
           resources:
             limits:
               cpu: "1"
               memory: "2Gi"
             requests:
               cpu: "500m"
               memory: "1Gi"
   ---
   apiVersion: v1
   kind: Service
   metadata:
     name: shardx
   spec:
     selector:
       app: shardx
     ports:
     - name: api
       port: 54868
       targetPort: 54868
     - name: web
       port: 54867
       targetPort: 54867
     type: LoadBalancer
   EOF

   # マニフェストを適用
   kubectl apply -f deployment.yaml
   ```

### AWS Lambda + API Gateway

サーバーレスアーキテクチャを使用してShardXの一部の機能をデプロイすることも可能です。

#### 注意事項

- Lambda関数の実行時間は最大15分に制限されています
- 長時間実行が必要なプロセスには適していません
- APIエンドポイントとして使用する場合に最適です

## Google Cloud Platform

Google Cloud Platform（GCP）は、コンピューティング、ストレージ、アプリケーション開発などのクラウドサービスを提供します。

### GKE (Google Kubernetes Engine)

GKEは、GoogleがホストするKubernetesのマネージドバージョンです。

#### デプロイ手順

1. **GKEクラスターの作成**:
   ```bash
   gcloud container clusters create shardx-cluster \
     --num-nodes=3 \
     --machine-type=e2-standard-2 \
     --region=us-central1
   ```

2. **Dockerイメージのビルドとプッシュ**:
   ```bash
   # Artifact Registryリポジトリを作成
   gcloud artifacts repositories create shardx \
     --repository-format=docker \
     --location=us-central1 \
     --description="ShardX Docker repository"

   # Dockerビルドとプッシュ
   gcloud builds submit --tag us-central1-docker.pkg.dev/$(gcloud config get-value project)/shardx/shardx:latest .
   ```

3. **Kubernetesマニフェストの適用**:
   ```bash
   # deployment.yamlファイルを作成
   cat > deployment.yaml << EOF
   apiVersion: apps/v1
   kind: Deployment
   metadata:
     name: shardx
   spec:
     replicas: 3
     selector:
       matchLabels:
         app: shardx
     template:
       metadata:
         labels:
           app: shardx
       spec:
         containers:
         - name: shardx
           image: us-central1-docker.pkg.dev/$(gcloud config get-value project)/shardx/shardx:latest
           ports:
           - containerPort: 54868
           - containerPort: 54867
           env:
           - name: RUST_LOG
             value: "info"
           - name: INITIAL_SHARDS
             value: "64"
           resources:
             limits:
               cpu: "1"
               memory: "2Gi"
             requests:
               cpu: "500m"
               memory: "1Gi"
   ---
   apiVersion: v1
   kind: Service
   metadata:
     name: shardx
   spec:
     selector:
       app: shardx
     ports:
     - name: api
       port: 54868
       targetPort: 54868
     - name: web
       port: 54867
       targetPort: 54867
     type: LoadBalancer
   EOF

   # マニフェストを適用
   kubectl apply -f deployment.yaml
   ```

### Google Cloud Run

Cloud Runは、コンテナ化されたアプリケーションをサーバーレスで実行するためのプラットフォームです。

#### デプロイ手順

1. **Dockerイメージのビルドとプッシュ**:
   ```bash
   # Artifact Registryリポジトリを作成
   gcloud artifacts repositories create shardx \
     --repository-format=docker \
     --location=us-central1 \
     --description="ShardX Docker repository"

   # Dockerビルドとプッシュ
   gcloud builds submit --tag us-central1-docker.pkg.dev/$(gcloud config get-value project)/shardx/shardx:latest .
   ```

2. **Cloud Runサービスのデプロイ**:
   ```bash
   gcloud run deploy shardx \
     --image us-central1-docker.pkg.dev/$(gcloud config get-value project)/shardx/shardx:latest \
     --platform managed \
     --region us-central1 \
     --allow-unauthenticated \
     --memory 2Gi \
     --cpu 1 \
     --port 54868 \
     --set-env-vars="RUST_LOG=info,INITIAL_SHARDS=64"
   ```

3. **Cloud Runサービスの確認**:
   ```bash
   gcloud run services describe shardx --platform managed --region us-central1
   ```

### Google Compute Engine

Compute Engineは、GCPの仮想マシンサービスです。

#### デプロイ手順

1. **VMインスタンスの作成**:
   ```bash
   gcloud compute instances create shardx-instance \
     --machine-type=e2-standard-2 \
     --image-family=debian-11 \
     --image-project=debian-cloud \
     --boot-disk-size=20GB \
     --tags=http-server,https-server
   ```

2. **ファイアウォールルールの作成**:
   ```bash
   gcloud compute firewall-rules create allow-shardx \
     --allow tcp:54867,tcp:54868 \
     --target-tags=http-server
   ```

3. **VMにSSH接続してShardXをインストール**:
   ```bash
   gcloud compute ssh shardx-instance

   # VMにDockerをインストール
   sudo apt-get update
   sudo apt-get install -y docker.io
   sudo systemctl enable docker
   sudo systemctl start docker

   # ShardXを実行
   sudo docker run -d -p 54867:54867 -p 54868:54868 --name shardx enablerdao/shardx:latest
   ```

## Microsoft Azure

Microsoft Azureは、マイクロソフトが提供するクラウドコンピューティングサービスです。

### AKS (Azure Kubernetes Service)

AKSは、Azureでホストされるマネージドなクラウドベースのコンテナオーケストレーションサービスです。

#### デプロイ手順

1. **AKSクラスターの作成**:
   ```bash
   # リソースグループの作成
   az group create --name shardx-rg --location eastus

   # AKSクラスターの作成
   az aks create \
     --resource-group shardx-rg \
     --name shardx-cluster \
     --node-count 3 \
     --enable-addons monitoring \
     --generate-ssh-keys
   ```

2. **クラスターの認証情報の取得**:
   ```bash
   az aks get-credentials --resource-group shardx-rg --name shardx-cluster
   ```

3. **Azure Container Registryの作成**:
   ```bash
   az acr create \
     --resource-group shardx-rg \
     --name shardxacr \
     --sku Basic
   ```

4. **ACRとAKSの統合**:
   ```bash
   # ACRのリソースIDを取得
   ACR_ID=$(az acr show --name shardxacr --resource-group shardx-rg --query "id" -o tsv)

   # AKSにACRへのアクセス権を付与
   az aks update \
     --name shardx-cluster \
     --resource-group shardx-rg \
     --attach-acr $ACR_ID
   ```

5. **Dockerイメージのビルドとプッシュ**:
   ```bash
   # ACRにログイン
   az acr login --name shardxacr

   # イメージをビルドしてプッシュ
   az acr build --registry shardxacr --image shardx:latest .
   ```

6. **Kubernetesマニフェストの適用**:
   ```bash
   # deployment.yamlファイルを作成
   cat > deployment.yaml << EOF
   apiVersion: apps/v1
   kind: Deployment
   metadata:
     name: shardx
   spec:
     replicas: 3
     selector:
       matchLabels:
         app: shardx
     template:
       metadata:
         labels:
           app: shardx
       spec:
         containers:
         - name: shardx
           image: shardxacr.azurecr.io/shardx:latest
           ports:
           - containerPort: 54868
           - containerPort: 54867
           env:
           - name: RUST_LOG
             value: "info"
           - name: INITIAL_SHARDS
             value: "64"
           resources:
             limits:
               cpu: "1"
               memory: "2Gi"
             requests:
               cpu: "500m"
               memory: "1Gi"
   ---
   apiVersion: v1
   kind: Service
   metadata:
     name: shardx
   spec:
     selector:
       app: shardx
     ports:
     - name: api
       port: 54868
       targetPort: 54868
     - name: web
       port: 54867
       targetPort: 54867
     type: LoadBalancer
   EOF

   # マニフェストを適用
   kubectl apply -f deployment.yaml
   ```

### Azure App Service

App Serviceは、Webアプリケーション、RESTful API、モバイルバックエンドをホストするためのフルマネージドプラットフォームです。

#### デプロイ手順

1. **App Serviceプランの作成**:
   ```bash
   az appservice plan create \
     --name shardx-plan \
     --resource-group shardx-rg \
     --sku P1V2 \
     --is-linux
   ```

2. **Webアプリの作成**:
   ```bash
   az webapp create \
     --resource-group shardx-rg \
     --plan shardx-plan \
     --name shardx-app \
     --deployment-container-image-name enablerdao/shardx:latest
   ```

3. **アプリケーション設定の構成**:
   ```bash
   az webapp config appsettings set \
     --resource-group shardx-rg \
     --name shardx-app \
     --settings WEBSITES_PORT=54868 RUST_LOG=info INITIAL_SHARDS=64
   ```

## オンプレミス環境

オンプレミス環境でShardXをデプロイする方法について説明します。

### 直接インストール

#### 前提条件

- Linux、macOS、またはWindowsサーバー
- Rust 1.60以上
- Git

#### デプロイ手順

1. **リポジトリのクローン**:
   ```bash
   git clone https://github.com/enablerdao/ShardX.git
   cd ShardX
   ```

2. **依存関係のインストール**:
   ```bash
   # Debian/Ubuntu
   sudo apt-get update
   sudo apt-get install -y build-essential libssl-dev pkg-config

   # RHEL/CentOS
   sudo yum install -y gcc openssl-devel

   # macOS
   brew install openssl
   ```

3. **ShardXのビルドと実行**:
   ```bash
   cargo build --release
   ./target/release/shardx --port 54868 --web-port 54867
   ```

4. **サービスとして設定（systemd）**:
   ```bash
   # shardx.serviceファイルを作成
   sudo cat > /etc/systemd/system/shardx.service << EOF
   [Unit]
   Description=ShardX Service
   After=network.target

   [Service]
   User=shardx
   WorkingDirectory=/opt/shardx
   ExecStart=/opt/shardx/target/release/shardx --port 54868 --web-port 54867
   Restart=on-failure
   Environment=RUST_LOG=info
   Environment=INITIAL_SHARDS=64

   [Install]
   WantedBy=multi-user.target
   EOF

   # サービスを有効化して起動
   sudo systemctl enable shardx
   sudo systemctl start shardx
   ```

### Docker Compose

#### 前提条件

- Docker
- Docker Compose

#### デプロイ手順

1. **docker-compose.ymlファイルの作成**:
   ```bash
   cat > docker-compose.yml << EOF
   version: '3'
   services:
     shardx:
       image: enablerdao/shardx:latest
       ports:
         - "54868:54868"
         - "54867:54867"
       environment:
         - RUST_LOG=info
         - INITIAL_SHARDS=64
       volumes:
         - shardx_data:/app/data
       restart: always

     redis:
       image: redis:alpine
       ports:
         - "6379:6379"
       volumes:
         - redis_data:/data
       restart: always

   volumes:
     shardx_data:
     redis_data:
   EOF
   ```

2. **Docker Composeの起動**:
   ```bash
   docker-compose up -d
   ```

## Kubernetes

Kubernetesを使用してShardXをデプロイする方法について説明します。

### Helmチャート

#### 前提条件

- Kubernetesクラスター
- Helm 3

#### デプロイ手順

1. **Helmチャートの作成**:
   ```bash
   # Helmチャートの作成
   mkdir -p shardx-chart/templates
   
   # Chart.yamlの作成
   cat > shardx-chart/Chart.yaml << EOF
   apiVersion: v2
   name: shardx
   description: A Helm chart for ShardX
   type: application
   version: 0.1.0
   appVersion: "1.0.0"
   EOF
   
   # values.yamlの作成
   cat > shardx-chart/values.yaml << EOF
   replicaCount: 3
   
   image:
     repository: enablerdao/shardx
     tag: latest
     pullPolicy: Always
   
   service:
     type: LoadBalancer
     apiPort: 54868
     webPort: 54867
   
   resources:
     limits:
       cpu: 1
       memory: 2Gi
     requests:
       cpu: 500m
       memory: 1Gi
   
   environment:
     RUST_LOG: info
     INITIAL_SHARDS: "64"
   EOF
   
   # deploymentテンプレートの作成
   cat > shardx-chart/templates/deployment.yaml << EOF
   apiVersion: apps/v1
   kind: Deployment
   metadata:
     name: {{ .Release.Name }}
     labels:
       app: {{ .Release.Name }}
   spec:
     replicas: {{ .Values.replicaCount }}
     selector:
       matchLabels:
         app: {{ .Release.Name }}
     template:
       metadata:
         labels:
           app: {{ .Release.Name }}
       spec:
         containers:
         - name: {{ .Chart.Name }}
           image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
           imagePullPolicy: {{ .Values.image.pullPolicy }}
           ports:
           - name: api
             containerPort: {{ .Values.service.apiPort }}
           - name: web
             containerPort: {{ .Values.service.webPort }}
           env:
           {{- range $key, $value := .Values.environment }}
           - name: {{ $key }}
             value: "{{ $value }}"
           {{- end }}
           resources:
             {{- toYaml .Values.resources | nindent 12 }}
   EOF
   
   # serviceテンプレートの作成
   cat > shardx-chart/templates/service.yaml << EOF
   apiVersion: v1
   kind: Service
   metadata:
     name: {{ .Release.Name }}
     labels:
       app: {{ .Release.Name }}
   spec:
     type: {{ .Values.service.type }}
     ports:
     - name: api
       port: {{ .Values.service.apiPort }}
       targetPort: {{ .Values.service.apiPort }}
     - name: web
       port: {{ .Values.service.webPort }}
       targetPort: {{ .Values.service.webPort }}
     selector:
       app: {{ .Release.Name }}
   EOF
   ```

2. **Helmチャートのインストール**:
   ```bash
   helm install shardx ./shardx-chart
   ```

3. **デプロイの確認**:
   ```bash
   kubectl get pods
   kubectl get services
   ```

## 高可用性構成

ShardXを高可用性（HA）構成でデプロイするための推奨事項です。

### マルチノード構成

#### アーキテクチャ

1. **フロントエンドレイヤー**:
   - ロードバランサー（AWS ALB、GCP Load Balancer、Azure Load Balancer）
   - 複数のWebインターフェースインスタンス

2. **APIレイヤー**:
   - 複数のAPIサーバーインスタンス
   - APIゲートウェイ（オプション）

3. **データレイヤー**:
   - 分散データストレージ
   - レプリケーション設定

#### 実装例（Kubernetes）

```yaml
# HA構成のKubernetesマニフェスト
apiVersion: apps/v1
kind: Deployment
metadata:
  name: shardx-api
spec:
  replicas: 5  # 複数のレプリカ
  selector:
    matchLabels:
      app: shardx
      tier: api
  template:
    metadata:
      labels:
        app: shardx
        tier: api
    spec:
      affinity:
        podAntiAffinity:  # 異なるノードに分散
          requiredDuringSchedulingIgnoredDuringExecution:
          - labelSelector:
              matchExpressions:
              - key: app
                operator: In
                values:
                - shardx
              - key: tier
                operator: In
                values:
                - api
            topologyKey: "kubernetes.io/hostname"
      containers:
      - name: shardx
        image: enablerdao/shardx:latest
        ports:
        - containerPort: 54868
        env:
        - name: RUST_LOG
          value: "info"
        - name: INITIAL_SHARDS
          value: "64"
        - name: REDIS_URL
          value: "redis-ha:6379"
        resources:
          limits:
            cpu: "1"
            memory: "2Gi"
          requests:
            cpu: "500m"
            memory: "1Gi"
        readinessProbe:
          httpGet:
            path: /api/v1/health
            port: 54868
          initialDelaySeconds: 5
          periodSeconds: 10
        livenessProbe:
          httpGet:
            path: /api/v1/health
            port: 54868
          initialDelaySeconds: 15
          periodSeconds: 20
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: shardx-web
spec:
  replicas: 3
  selector:
    matchLabels:
      app: shardx
      tier: web
  template:
    metadata:
      labels:
        app: shardx
        tier: web
    spec:
      containers:
      - name: shardx-web
        image: enablerdao/shardx:latest
        ports:
        - containerPort: 54867
        env:
        - name: WEB_ONLY
          value: "true"
        - name: API_URL
          value: "http://shardx-api:54868"
---
apiVersion: v1
kind: Service
metadata:
  name: shardx-api
spec:
  selector:
    app: shardx
    tier: api
  ports:
  - port: 54868
    targetPort: 54868
  type: ClusterIP
---
apiVersion: v1
kind: Service
metadata:
  name: shardx-web
spec:
  selector:
    app: shardx
    tier: web
  ports:
  - port: 54867
    targetPort: 54867
  type: LoadBalancer
```

### 障害復旧計画

1. **バックアップ戦略**:
   - 定期的なデータバックアップ
   - 複数のリージョンにバックアップを保存

2. **フェイルオーバー**:
   - 自動フェイルオーバーの設定
   - マルチリージョンデプロイメント

3. **監視とアラート**:
   - Prometheus + Grafanaによる監視
   - 障害検知のためのアラート設定

## セキュリティ推奨事項

ShardXをセキュアにデプロイするための推奨事項です。

### ネットワークセキュリティ

1. **TLS/SSL**:
   - すべての通信にTLS 1.3を使用
   - 自動証明書更新の設定

2. **ファイアウォール**:
   - 必要なポートのみを開放
   - IPベースのアクセス制限

3. **DDoS保護**:
   - CloudflareやAWS Shield等のDDoS保護サービスの利用
   - レート制限の実装

### アクセス制御

1. **認証**:
   - 強力なパスワードポリシー
   - 多要素認証（MFA）の実装

2. **認可**:
   - 最小権限の原則に基づくロールベースアクセス制御（RBAC）
   - APIキーのローテーション

3. **シークレット管理**:
   - AWS Secrets Manager、GCP Secret Manager、Azure Key Vaultなどの利用
   - 環境変数ではなくシークレット管理サービスを使用

## パフォーマンスチューニング

ShardXのパフォーマンスを最適化するための推奨事項です。

### リソース割り当て

1. **CPU**:
   - 小規模環境: 2 vCPU
   - 中規模環境: 4-8 vCPU
   - 大規模環境: 16+ vCPU

2. **メモリ**:
   - 小規模環境: 4 GB
   - 中規模環境: 8-16 GB
   - 大規模環境: 32+ GB

3. **ストレージ**:
   - SSDストレージの使用
   - 十分なIOPS確保

### シャード設定

1. **シャード数**:
   - 小規模環境: 32-64シャード
   - 中規模環境: 64-128シャード
   - 大規模環境: 128-256シャード

2. **シャードバランシング**:
   - 自動シャードバランシングの有効化
   - 定期的なリバランシングのスケジュール設定

### キャッシュ設定

1. **Redisキャッシュ**:
   - 適切なメモリ割り当て
   - 永続化設定の最適化

2. **キャッシュポリシー**:
   - 頻繁にアクセスされるデータのキャッシュ
   - 適切なTTL（Time To Live）設定

## まとめ

このガイドでは、ShardXをエンタープライズ環境にデプロイするための様々な方法と推奨事項を説明しました。環境やニーズに合わせて最適な方法を選択してください。

詳細な質問や支援が必要な場合は、[GitHubのIssue](https://github.com/enablerdao/ShardX/issues)を作成するか、[コミュニティフォーラム](https://forum.enablerdao.org)でご相談ください。