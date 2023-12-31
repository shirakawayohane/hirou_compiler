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

## TODOリスト
- 引数による関数のジェネリクス引数の自動推論
- 関数の戻り値の自動推論
- 型をGenericのままResolveできるようにする
- エラー箇所をわかるようにする
- 戻り値の型を検証
- 引数の数を検証
- indexがintであるかの検証
- structのフィールドにVoidを入れることは出来ないことの検証
- boolおよび演算の実装
- panic!, todo!, unreachable!の実装
