# VM state
list_compilation_level = 0
compiled_list = []

# VM General Dictionary
dictionary = {}

def add_to_dictionary(d, w, v):
    d[w] = v

def get_from_dictionary(d, w):
    return d[w]

# VM Stack
stack = []

def push_to_stack(s, v):
    s.append(v)

def pop_from_stack(s):
    return s.pop()

# VM Compiler and Executor
def tokenize(string):
    tokens = []
    word = ''
    isParsingString = False
    isSkippingComment = False
    for ch in string:
        if isParsingString:
            word = word + ch
            if ch == '\'':
                isParsingString = False
        elif isSkippingComment:
            if ch == '"':
                isSkippingComment = False
        else:
            if ch == '\'':
                # String token started
                isParsingString = True
                word = word + ch
            elif ch == '"':
                # Comment
                isSkippingComment = True
            elif ch != ' ' and ch != '\t' and ch != '\n':
                # Char belongs to a token that is not a string
                word = word + ch
            else:
                # Token separator, we have a complete token, store it
                if word != '':
                    #TODO: alias here
                    tokens.append(word)
                    word = ''
    #we reached the end, it is also a token separator
    if word != '':
        tokens.append(word)
    
    return tokens

def process_word(word):
    if word == '(':
        do_openpa()
    elif word == ')':
        do_closepa()
    else:
        if list_compilation_level > 0:
            compile_word(word)
        else:
            run_word(word)
    print("----------------------")

def compile_word(word):
    print("COMPILE WORD: " + word)
    if word == '(':
        do_openpa()
    elif word == ')':
        do_closepa()

def run_word(word):
    print("RUN WORD: " + word)
    #TODO: add other control words
    if word == '!':
        do_exclam()
    elif word == '@':
        do_at()
    else:
        do_normal(word)

# Word Executors

def do_exclam():
    print("CONTROL WORD: !")

def do_at():
    print("CONTROL WORD: @")

def do_openpa():
    print("CONTROL WORD: ( -> START COMPILING LIST")
    global list_compilation_level

    list_compilation_level = list_compilation_level + 1

def do_closepa():
    print("CONTROL WORD: ) -> END COMPILING LIST")
    global list_compilation_level
    global compiled_list

    list_compilation_level = list_compilation_level - 1
    if list_compilation_level == 0:
        print("PUSH LIST TO STACK: ")
        print(compiled_list)
        push_to_stack(stack, compiled_list)
        compiled_list = []

def do_normal(word):
    print("NORMAL WORD:  " + word)
    push_to_stack(stack, word)

# Type detectors

def is_int(word):
    try: 
        int(word)
        return True
    except ValueError:
        return False

def is_float(word):
    try: 
        float(word)
        return True
    except ValueError:
        return False

def is_string(word):
    return (word[0] == "'" and word[-1] == "'")

def is_bool(word):
    return (word == 'YES' or word == 'NO')

# VM Main Loop

def vm_loop(word_list):
    print("TOKENS = ")
    print(word_list)

    i = 0
    while i < len(word_list):
        word = word_list[i]
        process_word(word)
        i = i + 1
    print("\n\nFinished Executing. Stack = ")
    print(stack)

# User Program

program = """
"Això és un comentari"

10 20 + !
10 VAR-A @
( ) TUPLA @
( 10 20 ( A B C ) D )
HOLA 'Andreu Amic, Fins aviat!' !
"""

vm_loop(tokenize(program))
