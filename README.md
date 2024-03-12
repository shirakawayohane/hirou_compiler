```
 ________
< 疲労! >
 --------
        \   ^__^
         \  (><)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
```

# Hirou Compiler
I would have made a C LLVM front for learning, but it no longer looks like C at all. So, I have no choice but to try to make my own language.

# Requirements

## Mac, Linux
1. Install Rust
2. Install LLVM 16

Here is example installation of LLVM 16 on macOS
```
brew install llvm@16
echo export LLVM_SYS_160_PREFIX=/opt/homebrew/Cellar/llvm@16/16.0.6 >> ~/.bashrc
# If you use fish shell, use this
set -Ux LLVM_SYS_160_PREFIX /opt/homebrew/Cellar/llvm@16/16.0.6
```

## ロードマップ
- 基本的な言語機能の実装 <- 今ココ
    - リージョンベースのメモリ管理
- VSCodeでのシンタックスハイライト
- テストフレームワークの実装
- 標準ライブラリの実装

## TODOリスト
- boolおよび演算の実装
- 使用箇所からのジェネリクス引数の推論
- indexがintであるかの検証
- structのフィールドにVoidを入れることは出来ないことの検証
- panic!, todo!, unreachable!の実装
- annotationをOption<&ResolvedType>にできるか検討
- メモリ管理終わったら
    - ベクタリテラル []
    - マップリテラル {}
    - セットリテラル #{}
- 変数定義をS式に変える

変数定義こうしようと思ってる
```
(:= a 1)
(:= v  [] :- Vec<i32>
    v2 [] :- Vec<u32>)
```
