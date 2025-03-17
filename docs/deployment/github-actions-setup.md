# GitHub Actions設定ガイド

このガイドでは、ShardXプロジェクトのGitHub Actionsワークフローを設定する方法について説明します。

## DockerHubへの認証設定

GitHub ActionsからDockerHubにイメージをプッシュするには、DockerHubの認証情報をGitHub Secretsとして設定する必要があります。

### 1. DockerHubのアクセストークンを作成

1. [DockerHub](https://hub.docker.com/)にログイン
2. 右上のユーザーアイコンをクリックし、「Account Settings」を選択
3. 左側のメニューから「Security」を選択
4. 「New Access Token」をクリック
5. トークンの説明を入力し、アクセス権限を選択（通常は「Read & Write」）
6. 「Generate」をクリック
7. 生成されたトークンをコピー（このトークンは一度しか表示されないので注意）

### 2. GitHub Secretsに認証情報を設定

1. GitHubリポジトリのページに移動
2. 「Settings」タブをクリック
3. 左側のメニューから「Secrets and variables」→「Actions」を選択
4. 「New repository secret」をクリック
5. 以下の2つのシークレットを追加：
   - `DOCKERHUB_USERNAME`: DockerHubのユーザー名
   - `DOCKERHUB_TOKEN`: 先ほど生成したアクセストークン

### 3. ワークフローファイルの確認

`.github/workflows/docker-build.yml`ファイルに以下のような設定があることを確認します：

```yaml
- name: Login to DockerHub
  if: github.event_name != 'pull_request'
  uses: docker/login-action@v2
  with:
    username: ${{ secrets.DOCKERHUB_USERNAME }}
    password: ${{ secrets.DOCKERHUB_TOKEN }}
```

## ワークフローの手動実行

GitHub Actionsワークフローを手動で実行するには：

1. GitHubリポジトリのページに移動
2. 「Actions」タブをクリック
3. 左側のワークフローリストから「Build and Push Docker Image」を選択
4. 「Run workflow」ボタンをクリック
5. 必要に応じてDockerイメージのタグを指定（デフォルトは「latest」）
6. 「Run workflow」をクリック

## トラブルシューティング

### DockerHub認証エラー

```
failed to authorize: failed to fetch oauth token: unexpected status from GET request to https://auth.docker.io/token?scope=repository%3Aenablerdao%2Fshardx%3Apull%2Cpush&service=registry.docker.io: 401 Unauthorized
```

このエラーが発生した場合：

1. GitHub Secretsが正しく設定されているか確認
2. DockerHubのアクセストークンが有効か確認
3. DockerHubのユーザー名が正確か確認
4. DockerHubのリポジトリ名が正確か確認（大文字小文字を区別）

### イメージのビルドエラー

Dockerイメージのビルド中にエラーが発生した場合：

1. ワークフローの実行ログを確認
2. Dockerfileに問題がないか確認
3. 必要な依存関係がすべて含まれているか確認

## 参考リンク

- [GitHub Actions公式ドキュメント](https://docs.github.com/en/actions)
- [Docker GitHub Actions公式ドキュメント](https://docs.docker.com/ci-cd/github-actions/)
- [DockerHubアクセストークンの作成](https://docs.docker.com/docker-hub/access-tokens/)