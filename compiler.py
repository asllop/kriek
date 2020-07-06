# Primitives

## Helpers

### Guarantee that word is a primite type, if it's defined, get content
def word_or_value(word):
    if is_int(word) or is_float(word) or is_bool(word) or is_string(word) or is_list(word):
        return word
    else:
        if word in dictionary:
            return dictionary[word]
        else:
            return None

## Integer

### Message '+' of int
def int_plus(word):
    try: 
        argA = int(word)
        argB = int(word_or_value(pop_from_stack()))
        push_to_stack(str(argA + argB))
    except Exception:
        #TODO: ERROR
        print("ERROR: int_plus: error  converting to int")

### Message '-' of int
def int_minus(word):
    try: 
        argA = int(word)
        argB = int(word_or_value(pop_from_stack()))
        push_to_stack(str(argB - argA))
    except Exception:
        #TODO: ERROR
        print("ERROR: int_minus: error converting to int")

## List

### Message 'SIZE' of list
def list_size(word):
    push_to_stack(str(len(word)))

# VM state

## Compilated Lists
list_compilation_level = 0
compiled_list = []

def add_to_list(word):
    compiled_list.append(word)

## Dictionaries

### Global Dictionary
global_dictionary = {
    "INTEGER": "0",
    "FLOAT": "0.0",
    "STRING": "''",
    "BOOLEAN": "NO",
    "LIST": []
}

### Local Word Dictionaries
word_dictionaries = {
    "INTEGER": {
        '+': int_plus,
        '-': int_minus
    },
    "FLOAT": {},
    "STRING": {},
    "BOOLEAN": {},
    "LIST": {
        "SIZE": list_size
    }
}

### Current Dictionary
dictionary = global_dictionary

def add_to_dictionary(w, v):
    dictionary[w] = v

def get_from_dictionary(w):
    return dictionary[w]

def init_word_dictionary(w):
    word_dictionaries[w] = {}

## Stack
stack = []

def push_to_stack(v):
    stack.append(v)

def pop_from_stack():
    return stack.pop()

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
    add_to_list(word)

def run_word(word):
    print("RUN WORD: " + word)
    if word == '!':
        do_exclam()
    elif word == '@':
        do_at()
    elif word == ':':
        do_colon()
    elif word == '~':
        do_tilde()
    else:
        do_normal(word)

## Word Executors

def exec_word(recv, msg, d):
    if msg in d:
        v = d[msg]
        if callable(v):
            v(recv)
        else:
            #TODO: execute v, is defined word
            print("IS DEFINED: " + str(v))
    else:
        return False

    return True

def do_exclam():
    print("CONTROL WORD: !")
    msg = pop_from_stack()
    recv = pop_from_stack()

    word_dict = None
    type_dict = None

    # word dictionary exist for word, get its word dictionary
    if recv in word_dictionaries:
        word_dict = word_dictionaries[recv]

    # get its primitive type dictionary
    content = word_or_value(recv)

    if is_int(content):
        type_dict = word_dictionaries['INTEGER']
    elif is_float(content):
        type_dict = word_dictionaries['FLOAT']
    elif is_string(content):
        type_dict = word_dictionaries['STRING']
    elif is_bool(content):
        type_dict = word_dictionaries['BOOLEAN']
    elif is_list(content):
        type_dict = word_dictionaries['LIST']
    else:
        #TODO: error
        print("ERROR: Unknown type")
        return
    
    # If word dictionary exist, execute it
    did_exec_msg = False
    if word_dict != None:
        did_exec_msg = exec_word(recv, msg, word_dict)
    
    # If couldn't find the msg in word, try with its primitive type
    if did_exec_msg == False:
        did_exec_msg = exec_word(content, msg, type_dict)

    if did_exec_msg == False:
        #TODO: error
        print("ERROR: no message " + str(msg) + " in word " + str(recv))


def do_at():
    print("CONTROL WORD: @")
    word = pop_from_stack()
    value = word_or_value(pop_from_stack())
    if value != None:
        add_to_dictionary(word, value)
    else:
        #TODO: ERROR
        print("ERROR: value word doesn't exist")
        return

def do_colon():
    print("CONTROL WORD: :")
    global dictionary
    # Current dictionary is stack word's dictionary
    word = pop_from_stack()
    if word in dictionary:
        if word not in word_dictionaries:
            init_word_dictionary(word)
        d = word_dictionaries[word]
        dictionary = d
    else:
        #TODO: error
        print("ERROR: trying to get dictionary from a word that doesn't exist")
        return
    
def do_tilde():
    print("CONTROL WORD: ~")
    global dictionary
    dictionary = global_dictionary

def do_openpa():
    print("CONTROL WORD: ( -> START COMPILING LIST")
    global list_compilation_level

    if list_compilation_level > 0:
        add_to_list('(')

    list_compilation_level = list_compilation_level + 1

def do_closepa():
    print("CONTROL WORD: ) -> END COMPILING LIST")
    global list_compilation_level
    global compiled_list

    list_compilation_level = list_compilation_level - 1

    if list_compilation_level > 0:
        add_to_list(')')
    else:
        print("PUSH LIST TO STACK: ")
        print(compiled_list)
        push_to_stack(compiled_list)
        compiled_list = []

def do_normal(word):
    print("NORMAL WORD:  " + word)
    push_to_stack(word)

## Type detectors

def is_int(word):
    try: 
        int(word)
        return True
    except Exception:
        return False

def is_float(word):
    try: 
        float(word)
        return True
    except Exception:
        return False

def is_string(word):
    return (word[0] == "'" and word[-1] == "'")

def is_bool(word):
    return (word == 'YES' or word == 'NO')

def is_list(word):
    return isinstance(word, list)

# VM Main Loop

def vm_loop(word_list):
    print("TOKENS = ")
    print(word_list)

    i = 0
    while i < len(word_list):
        word = word_list[i]
        process_word(word)
        i = i + 1
    
    print("\n\nFinished Executing.\n")
    print("Stack = ")
    print(stack)
    print()
    print("Global Dictionary = ")
    print(global_dictionary)
    print()
    print("Word Dictionaries = ")
    print(word_dictionaries)

# User Program

program = """
"Això és un comentari"

10 20 + !

10 VAR-A @
20 VAR-B @
30 VAR-C @
40 VAR-D @

15 VAR-A - !
VAR-A 15 - !
VAR-A VAR-D + !

( 5 15 ( VAR-A VARB-B ( VAR-C ) ) VAR-D )
'Fins aviat amics!'

( VAR-A VAR-B VAR-C VAR-D ) TUPLA @
TUPLA :
    66 X @
    99 Y @
    ( 99 1 + ! ) SUMA-100 @
~

"Operate inside a word dictionary"
TUPLA : Y X - ! ~

10 GLOBAL-X @
20 GLOBAL-Y @

TUPLA SIZE !
TUPLA X !
TUPLA SUMA-100 !
"""

vm_loop(tokenize(program))
