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
2. Install LLVM 14

Here is example installation of LLVM 14 on macOS
```
brew install llvm@14
echo export LLVM_SYS_140_PREFIX=/usr/homebrew/opt/llvm@14 >> ~/.bashrc
```

## TODOリスト
- エラー箇所をわかるようにする
- 戻り値の型
- 引数の型
- 引数の数
- indexがintである
- IRで構造体の型を読めるようにする
- structのフィールドにVoidを入れることは出来ない
