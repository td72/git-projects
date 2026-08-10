# git-projects

fzf を使ってリポジトリを選択し、パスを出力する CLI ツール。

## 依存

- [fzf](https://github.com/junegunn/fzf)

## インストール

```sh
cargo install --git https://github.com/td72/git-projects
```

## 使い方

シェルの設定ファイルに以下を追加:

```sh
function cd-git-projects() {
  local dir=$(git-projects choice)
  if [ -n "$dir" ]; then
    cd "$dir"
  fi
}
```

### choice コマンド

`GIT_PROJECTS_TARGETS` に一致するリポジトリのみ表示:

```sh
git-projects choice
git-projects c  # alias
```

全リポジトリを表示:

```sh
git-projects choice all
git-projects c all
```

## 履歴

選択したリポジトリは `$XDG_DATA_HOME/git-projects/history`（未設定なら `~/.local/share/git-projects/history`）に記録され、次回以降は最近選んだ順に上位へ並ぶ。保持は最新 100 件。

## 終了コード

| コード | 意味 |
|---|---|
| 0 | 選択成功、またはユーザーがキャンセル（何も出力しない） |
| 1 | エラー（fzf が見つからない、など。メッセージは stderr へ） |
| 2 | 引数が不正 |

## 設定

### リポジトリルート

`git config --global ghq.root` の値を使用。未設定の場合は `$HOME/src` にフォールバック。

### ターゲットフィルタ

環境変数 `GIT_PROJECTS_TARGETS` にコロン区切りで指定。パスに含まれるリポジトリのみ表示される。

```sh
export GIT_PROJECTS_TARGETS="github.com/td72:github.com/myorg"
```

未設定の場合は全リポジトリを表示。
