# Kriek
_An embeddable scripting language with the best of Forth, SmallTalk and Lisp_

**NOTE: The language definition and the VM are still work in progress.**

Kriek is a small, flexible, powerful and easily embeddable programming language designed to work as in-app scripting platform. It's strongly influenced by **Forth**, **SmallTalk** and **Lisp** and it takes the main ideas from those languages:

- As Forth, it's based on words and uses a stack to pass data between words. Also as Forth, it uses a dictionary to store words, but Kriek extends this idea, allowing words to have its own private dictionary inside, where other words can be defined.
- As SmallTalk, code is executed by passing messages to objects (in Kriek, words). Messages in Kriek are other words, that are defined inside receiver's dictionary.
- As Lisp, it uses lists as the way to define modules as well as data structures. In Kriek words are actually lists of other words.
