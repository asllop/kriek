import sys

# Primitives

#TODO - Implement messages that can't be created in Kriek, only in native code:
# - LIST messages: LOOP(?), SET, GET, ADD, DEL, DO(?)
# - STRING messages: SIZE, SET, GET, ADD, DEL
# - BOOLEAN:
# - INT & FLOAT: STR (convert to string)

#TODO - Implement messages in Kriek:
# - STRING: + (concatenate), SUB (substring), FIND, SPLIT.
# - LIST: MAP, REDUCE

#TODO: implement control words: ^ (copy), $ (return)

#TODO:
# - Think about $ and the inner levels

## Integer

### Message '+' of int
def int_plus(word):
    try: 
        argA = int(word)
        argB = int(word_or_value(pop_from_stack()))
        push_to_stack(str(argA + argB))
    except Exception:
        fail("ERROR: int_plus: error  converting to int")

### Message '-' of int
def int_minus(word):
    try: 
        argA = int(word)
        argB = int(word_or_value(pop_from_stack()))
        push_to_stack(str(argB - argA))
    except Exception:
        fail("ERROR: int_minus: error converting to int")

### Message '*' of int
def int_mul(word):
    try: 
        argA = int(word)
        argB = int(word_or_value(pop_from_stack()))
        push_to_stack(str(argB * argA))
    except Exception:
        fail("ERROR: int_mul: error converting to int")

### Message '/' of int
def int_div(word):
    try: 
        argA = int(word)
        argB = int(word_or_value(pop_from_stack()))
        push_to_stack(str(argB // argA))
    except Exception:
        fail("ERROR: int_div: error converting to int")


### Message '%' of int
def int_mod(word):
    try: 
        argA = int(word)
        argB = int(word_or_value(pop_from_stack()))
        push_to_stack(str(argB % argA))
    except Exception:
        fail("ERROR: int_mod: error converting to int")

### Message '>' of int
def int_bigger(word):
    try: 
        argA = int(word)
        argB = int(word_or_value(pop_from_stack()))
        if argB > argA:
            push_to_stack('YES')
        else:
            push_to_stack('NO')
    except Exception:
        fail("ERROR: int_bigger: error converting to int")

### Message '=' of int
def int_equal(word):
    try: 
        argA = int(word)
        argB = int(word_or_value(pop_from_stack()))
        if argB == argA:
            push_to_stack('YES')
        else:
            push_to_stack('NO')
    except Exception:
        fail("ERROR: int_equal: error converting to int")

## Float

### Message '+' of float
def float_plus(word):
    try: 
        argA = float(word)
        argB = float(word_or_value(pop_from_stack()))
        push_to_stack(str(argA + argB))
    except Exception:
        fail("ERROR: float_plus: error converting to float")

### Message '-' of float
def float_minus(word):
    try: 
        argA = float(word)
        argB = float(word_or_value(pop_from_stack()))
        push_to_stack(str(argB - argA))
    except Exception:
        fail("ERROR: float_minus: error converting to float")

### Message '*' of float
def float_mul(word):
    try: 
        argA = float(word)
        argB = float(word_or_value(pop_from_stack()))
        push_to_stack(str(argB * argA))
    except Exception:
        fail("ERROR: float_mul: error converting to float")

### Message '/' of float
def float_div(word):
    try: 
        argA = float(word)
        argB = float(word_or_value(pop_from_stack()))
        push_to_stack(str(argB / argA))
    except Exception:
        fail("ERROR: float_div: error converting to float")

### Message '>' of float
def float_bigger(word):
    try: 
        argA = float(word)
        argB = float(word_or_value(pop_from_stack()))
        if argB > argA:
            push_to_stack('YES')
        else:
            push_to_stack('NO')
    except Exception:
        fail("ERROR: float_bigger: error converting to float")

### Message '=' of float
def float_equal(word):
    try: 
        argA = float(word)
        argB = float(word_or_value(pop_from_stack()))
        if argB == argA:
            push_to_stack('YES')
        else:
            push_to_stack('NO')
    except Exception:
        fail("ERROR: float_equal: error converting to float")

## Boolean

### Message 'IF' of boolean
def bool_if(word):
    block = pop_from_stack()
    if word == 'YES':
        vm_loop(block)

### Message 'AND' of boolean
def bool_and(word):
    b = pop_from_stack()
    if not is_bool(b):
        fail("ERROR: argument not a boolean")
    if word == 'YES' and b == 'YES':
        push_to_stack('YES')
    else:
        push_to_stack('NO')

### Message 'OR' of boolean
def bool_or(word):
    b = pop_from_stack()
    if not is_bool(b):
        fail("ERROR: argument not a boolean")
    if word == 'NO' and b == 'NO':
        push_to_stack('NO')
    else:
        push_to_stack('YES')

### Message 'NOT' of boolean
def bool_not(word):
    if word == 'YES':
        push_to_stack('NO')
    else:
        push_to_stack('YES')

### Message 'IF-ELSE' of boolean
def bool_ifelse(word):
    no_block = pop_from_stack()
    yes_block = pop_from_stack()
    if word == 'YES':
        vm_loop(yes_block)
    else:
        vm_loop(no_block)

## List

### Message 'SIZE' of list
def list_size(word):
    push_to_stack(str(len(word)))

### Message 'WHILE' of list
def list_while(block):
    condition = pop_from_stack()
    while True:
        vm_loop(condition)
        b = pop_from_stack()
        if not is_bool(b):
            fail("ERROR: WHILE condition not a boolean")
        if b == 'YES':
            vm_loop(block)
        else:
            return

# VM state

## Compilated Lists
list_compilation_level = 0
compiled_list = []
receiver_stack = []

def add_to_list(word):
    compiled_list.append(word)

def push_to_receiver_stack(w):
    receiver_stack.append(w)

def pop_from_receiver_stack():
    return receiver_stack.pop()

def get_current_receiver():
    return receiver_stack[-1]

## Dictionaries

### Global Dictionary
global_dictionary = {
    "INTEGER": ["0", {
        '+': int_plus,
        '-': int_minus,
        '*': int_mul,
        '/': int_div,
        '%': int_mod,
        '>': int_bigger,
        '=': int_equal
    }],
    "FLOAT": ["0.0", {
        '+': float_plus,
        '-': float_minus,
        '*': float_mul,
        '/': float_div,
        '>': float_bigger,
        '=': float_equal
    }],
    "STRING": ["''", {}],
    "BOOLEAN": ["NO", {
        "IF": bool_if,
        "IF-ELSE": bool_ifelse,
        "AND": bool_and,
        "OR": bool_or,
        "NOT": bool_not
    }],
    "LIST": [[], {
        "SIZE": list_size,
        "WHILE": list_while
    }]
}

### Stack of dictionaries
dictionary_stack = [global_dictionary]

### Current Dictionary
dictionary = dictionary_stack[-1]

def push_to_dictionary_stack(d):
    global dictionary
    dictionary_stack.append(d)
    dictionary = dictionary_stack[-1]

def pop_from_dictionary_stack():
    global dictionary
    if len(dictionary_stack) > 0:
        dictionary_stack.pop()
        dictionary = dictionary_stack[-1]
    else:
        fail("ERROR: dictionary stack underflow")

### Functions to operate with dictionaries and words
def add_to_dictionary(w, v):
    if is_list(w):
        w = repr(w) #Lists are not hashable in Python, we have to convert to string
    dictionary[w] = [v, {}]

def get_word_value_from_dictionary(w, d):
    if w in d:
        t = d[w]
        return t[0]
    else:
        return None

def get_word_value(w):
    return get_word_value_from_dictionary(w, dictionary)

def get_word_dictionary_using_dictionary(w, d):
    if is_list(w):
        w = repr(w) #Lists are not hashable in Python, we have to convert to string
    if w in d:
        t = d[w]
        return t[1]
    else:
        return None

def get_word_dictionary(w):
    return get_word_dictionary_using_dictionary(w, dictionary)

def exist_word_in_word_dictionary(w, recv_w):
    d = get_word_dictionary(recv_w)
    if d == None:
        return False
    else:
        return w in d

def move_to_word_dictionary(w):
    d = get_word_dictionary(w)
    if d != None:
        push_to_dictionary_stack(d)
    else:
        fail("ERROR: word " + w + " not found in current dictionary")

def go_back_to_previous_dictionary():
    pop_from_dictionary_stack()

def find_word_in_dictionary_stack(w):
    for d in reversed(dictionary_stack):
        # find w inside d
        new_d = get_word_dictionary_using_dictionary(w, d)
        if new_d != None:
            return d
    return None

def word_or_value(word):
    if is_int(word) or is_float(word) or is_bool(word) or is_string(word) or is_list(word):
        return word
    else:
        return get_word_value(word)

## Stack
stack = []

def push_to_stack(v):
    stack.append(v)

def pop_from_stack():
    if len(stack) > 0:
        return stack.pop()
    else:
        fail("ERROR: stack underflow")

# VM Compiler and Executor
aliases = {}

def compile(string):
    tokens = tokenize(string)
    # find alias
    clean_tokens = []
    i = 0
    while i < len(tokens):
        w = tokens[i]
        definition = []
        alias_word = None
        if w == '[':
            i = i + 1
            while tokens[i] != ']':
                definition.append(tokens[i])
                i = i + 1
            i = i + 1
            alias_word = tokens[i]
            aliases[alias_word] = definition
        else:
            clean_tokens.append(w)
        
        i = i + 1
    # free mem
    del tokens
    # substitute aliases by definitions
    final_tokens = []
    i = 0
    while i < len(clean_tokens):
        w = clean_tokens[i]
        if w in aliases:
            final_tokens.extend(aliases[w])
        else:
            final_tokens.append(w)
        i = i + 1
    
    return final_tokens

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
    elif word == '.':
        do_dot()
    elif word == ',':
        do_comma()
    elif word == '\\d':
        do_stack_duplicate()
    elif word == '\\s':
        do_stack_swap()
    elif word == '\\r':
        do_stack_remove()
    elif word == '\\c':
        do_stack_copy()
    elif word == '\\e':
        do_stack_extract()
    else:
        do_normal(word)

## Word Executors

def exec_word(recv, msg, d):
    if msg in d:
        v = d[msg]
        if callable(v):
            v(recv)
        else:
            w = v[0]
            if is_list(w):
                vm_loop(w)
            else:
                vm_loop([w])
    else:
        return False
    
    return True

def exec_in_primitive(content, msg):
    type_dict = None
    if content != None:
        # primitives are defined in the global dictionary
        if is_int(content):
            type_dict = global_dictionary['INTEGER'][1]
        elif is_float(content):
            type_dict = global_dictionary['FLOAT'][1]
        elif is_string(content):
            type_dict = global_dictionary['STRING'][1]
        elif is_bool(content):
            type_dict = global_dictionary['BOOLEAN'][1]
        elif is_list(content):
            type_dict = global_dictionary['LIST'][1]
        else:
            fail("ERROR: Unknown type")
        
        r = exec_word(content, msg, type_dict)
        return r
    else:
        return False

def do_exclam():
    print("CONTROL WORD: !")
    msg = pop_from_stack()
    recv = pop_from_stack()

    push_to_receiver_stack(recv)

    r = False

    #TODO: rework this structure:
    # Try to find word in all the dictionary stack starting by top
    # If not found, error
    # If found, try to exec msg
    # If msg fails, try with primitive
    # If it fails, error

    if exist_word_in_word_dictionary(msg, recv):
        # msg exist inside recv, execute it
        d = get_word_dictionary(recv)
        r = exec_word(recv, msg, d)
    else:
        # msg doesn't exist inside recv, try with its content primitive
        content = word_or_value(recv)
        r = exec_in_primitive(content, msg)

        if r != True:
            # word still doesn't exist, go down in the dictionary stack until reach the base or find it
            d = find_word_in_dictionary_stack(recv)
            if d != None:
                w_dict = get_word_dictionary_using_dictionary(recv, d)
                r = exec_word(recv, msg, w_dict)
                if r == False:
                    content = get_word_value_from_dictionary(recv, d)
                    r = exec_in_primitive(content, msg)
            else:
                fail("ERROR: Word " + recv + " doesn't exist in dictionary stack")

    if r == False:
        fail("ERROR: no message " + str(msg) + " in word " + str(recv))

    pop_from_receiver_stack()

def do_at():
    print("CONTROL WORD: @")
    word = pop_from_stack()
    value = word_or_value(pop_from_stack())
    if value != None:
        add_to_dictionary(word, value)
    else:
        fail("ERROR: value word doesn't exist")

#TODO: find in dictionary stack?
def do_colon():
    print("CONTROL WORD: :")
    word = pop_from_stack()
    move_to_word_dictionary(word)

def do_tilde():
    print("CONTROL WORD: ~")
    go_back_to_previous_dictionary()

def do_dot():
    print("CONTROL WORD: .")
    push_to_stack(get_current_receiver())

def do_comma():
    print("CONTROL WORD: ,")
    w = pop_from_stack()
    v = word_or_value(w)
    push_to_stack(v)

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

def do_stack_duplicate():
    print("STACK DUPLICATE")
    x = pop_from_stack()
    push_to_stack(x)
    push_to_stack(x)

def do_stack_swap():
    print("STACK SWAP")
    x = pop_from_stack()
    y = pop_from_stack()
    push_to_stack(x)
    push_to_stack(y)

def do_stack_remove():
    print("STACK REMOVE")
    pop_from_stack()

def do_stack_copy():
    print("STACK COPY")
    n = pop_from_stack()
    if is_int(n):
        n = int(n)
        if len(stack) > n:
            push_to_stack(stack[-1*(n + 1)])
        else:
            fail("ERROR: index out of stack size")
    else:
        fail("ERROR: index not an integer")

def do_stack_extract():
    print("STACK EXTRACT")
    n = pop_from_stack()
    if is_int(n):
        n = int(n)
        if len(stack) > n:
            x = stack[-1*(n + 1)]
            del stack[-1*(n + 1)]
            push_to_stack(x)
        else:
            fail("ERROR: index out of stack size")
    else:
        fail("ERROR: index not an integer")

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
    if len(word) >= 2:
        return (word[0] == "'" and word[-1] == "'")
    else:
        return False

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
    
    print("END VM LOOP")

def run_krfile(f):
    with open(f, 'r') as krfile:
        program = krfile.read()
    vm_loop(compile(program))

def fail(msg):
    print(msg)
    exit(1)

# Execute lexicons

run_krfile('lexicon/essential.kr')

# User Program

if len(sys.argv) != 2:
    fail("\n\nUSAGE:\n$ kr-run FILE\n\n")

run_krfile(sys.argv[1])

print("\n\nFinished Executing.\n")
print("Stack = ")
print(stack)
print()
print("Global Dictionary = ")
print(global_dictionary)
print()
print("Receiver Stack = ")
print(receiver_stack)
print()
print("Aliases = ")
print(aliases)
print()
print("Dictionary Stack (" + str(len(dictionary_stack)) + ") = ")
print(dictionary_stack)
print()