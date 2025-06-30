# numbers-game

チェッカーと独自の要素を組み合わせたオリジナルのボードゲームです。

## 実行

実行前に [Rust](https://www.rust-lang.org/) と [Node.js](https://nodejs.org/) のインストールが必要です。

ゲームサーバー:

```
cargo r --bin numbers-server
```

フロントエンド:

```
cd apps/frontend && npm i && npm run dev
```

## 開発

型の生成:

```
# まだ済んでいなければ
cd apps/frontend && npm i && cd ../..

cargo r --bin numbers-comm-types
```

## 計画

https://hackmd.io/@yuto0214w/r1YJI5JBex

## 感謝

- @kagesakura
  - コードに関する助言やゲームデザインの相談等
