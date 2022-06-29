# Kriek
_Lexicon Oriented Programming Language_

## Preface

The Forth programming language is probably one of the most idiosyncratic and powerful Memes in computing history. When it appeared in the 70’s it was revolutionary, and all the underlying concepts still remain valuable and contemporary, but some of its details may look a bit aged if we inspect it with the optics of present state-of-the-art computer programming. A handful of nice projects appeared over the years, Factor being the most prominent, following the path traced by Forth and adding modern features. These languages include things like classes, functional operators, garbage collection, complex data primitives, etc. But in my opinion, they lose part of the essence of Forth. In contrast, **Kriek** pursues simplicity, and instead of borrowing concepts from other programming philosophies, it rests upon the set of tools and ideas that originally backed Forth: It extends and delves into the notion of **Lexicon**, which proved to be powerful, compact, and flexible. For this reason, I call Kriek a **Lexicon Oriented Programming Language**.

Kriek received influences from all the programming languages I ever used (and liked), but clearly the strongest ones are:

- Forth: 90% of it!
- Factor: vocabularies inspired lexicons.
- Rust: supertraits inspired lexicon unions.
- Racket: lists inspired nested stacks.
- Swift: automatic reference counting.

## Introduction

Kriek is a [stack-based](https://en.wikipedia.org/wiki/Stack-oriented_programming), [concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language) programming language, designed to be easily embedded into other applications. For this purpose we provide an implementation in Rust (no_std), and comming soon implementations in Kotlin and Swift. Having and maintaining multiple implementations is possible because the Kriek interpreter is pretty small and simple, around 1k LOC, and most of the complexity is in the **prelude**, that contains many core features like variables, if-else statements, loops and arrays, to name some. In other languages these things are part of the compiler/interpeter but in Kriek are just normal words writen in Kriek itself.
