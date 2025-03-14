# ShardX Kubernetes デプロイガイド

このディレクトリには、ShardXをKubernetesクラスタにデプロイするためのマニフェストファイルが含まれています。

## 前提条件

- Kubernetesクラスタ（バージョン1.19以上）
- kubectl（クラスタと通信するためのコマンドラインツール）
- kustomize（オプション、ただし推奨）

## クイックスタート

### 1. 名前空間の作成

```bash
kubectl create namespace shardx
```

### 2. マニフェストの適用

kustomizeを使用する場合：

```bash
kubectl apply -k .
```

kustomizeを使用しない場合：

```bash
kubectl apply -f storage.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
```

### 3. デプロイの確認

```bash
kubectl get pods -n shardx
kubectl get services -n shardx
```

## アクセス方法

### ポートフォワーディングを使用する場合

```bash
# Webインターフェース
kubectl port-forward -n shardx svc/shardx-web 8080:80

# API
kubectl port-forward -n shardx svc/shardx-node 8081:54868
```

これにより、以下のURLでアクセスできます：
- Webインターフェース: http://localhost:8080
- API: http://localhost:8081

### Ingressを使用する場合

Ingressコントローラーがクラスタにインストールされていることを確認し、必要に応じてservice.yamlのホスト名を変更してください。

## 設定のカスタマイズ

kustomization.yamlを編集することで、以下の設定をカスタマイズできます：

- イメージタグ
- 環境変数
- リソース制限
- レプリカ数

例えば、レプリカ数を変更するには：

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - deployment.yaml
  - service.yaml
  - storage.yaml

namespace: shardx

commonLabels:
  app: shardx

images:
  - name: enablerdao/shardx
    newTag: v1.0.0  # タグを変更

patches:
  - target:
      kind: Deployment
      name: shardx-node
    patch: |
      - op: replace
        path: /spec/replicas
        value: 3  # レプリカ数を3に変更
```

## 本番環境での考慮事項

- **ストレージ**: 本番環境では、適切なストレージクラスを使用してください。
- **セキュリティ**: NetworkPolicyを追加して、ポッド間の通信を制限することを検討してください。
- **モニタリング**: Prometheus/Grafanaを使用して、クラスタとアプリケーションのメトリクスを監視することをお勧めします。
- **バックアップ**: 定期的なデータバックアップを設定してください。

## トラブルシューティング

### ポッドが起動しない場合

```bash
kubectl describe pod -n shardx <pod-name>
kubectl logs -n shardx <pod-name>
```

### サービスにアクセスできない場合

```bash
kubectl get endpoints -n shardx
kubectl describe service -n shardx <service-name>
```