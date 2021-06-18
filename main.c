#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum TokenType {
    TOK_NUM,
    TOK_STR,
    TOK_SYM,
    TOK_ERR,
    TOK_END,
};

#define TOK_CAPACITY 255
#define OUT_BUF_CAPACITY 255

struct TokenArray {
    size_t len;
    size_t cap;
    enum TokenType* types;
    size_t* starts;
    size_t* ends;
};

bool tok_array_full(struct TokenArray tokens)
{
    return tokens.len >= tokens.cap;
}

bool tok_array_push(struct TokenArray* tokens, enum TokenType type,
    size_t start, size_t end)
{
    if (tok_array_full(*tokens)) {
        return false;
    }
    tokens->types[tokens->len] = type;
    tokens->starts[tokens->len] = start;
    tokens->ends[tokens->len] = end;
    tokens->len++;
    return true;
}

bool tokenize_str(struct TokenArray* tokens, char* in, size_t len)
{
    bool in_tok = false;
    bool in_str = false;
    bool in_num = false;
    size_t last_start = 0;
    for (size_t i = 0; i <= len; i++) {
        if (in[i] == '"') {
            in_str = !in_str;
            in_tok = in_str;
            if (in_str) {
                last_start = i;
            } else {
                if (!tok_array_push(tokens, TOK_STR, last_start, i + 1)) {
                    return false;
                }
            }
            continue;
        }
        bool is_sep = !in_str && (in[i] == ' ' || in[i] == '\t' || in[i] == '\0');
        if (is_sep && in_tok) {
            if (!tok_array_push(tokens, in_num ? TOK_NUM : TOK_SYM, last_start, i)) {
                return false;
            }
            in_num = false;
        }
        if (!is_sep && !in_tok) {
            last_start = i;
            in_num = in[i] >= '0' && in[i] <= '9';
        }
        in_tok = !is_sep;
    }
    if (in_str) {
        tok_array_push(tokens, TOK_ERR, last_start, len);
    }
    tok_array_push(tokens, TOK_END, len, len + 1);
    return true;
}

struct BytesBuffer {
    size_t len;
    size_t cap;
    int8_t* data;
};

bool buf_write_str(struct BytesBuffer* buf, char* str, size_t len)
{
    if (len >= buf->cap) {
        return false;
    }
    memcpy(&buf->data[buf->len], str, len);
    buf->len += len;
    return true;
}

bool buf_write_str_range(struct BytesBuffer* buf, char* str, size_t start,
    size_t end)
{
    size_t len = end - start;
    return buf_write_str(buf, &str[start], len);
}

bool buf_write_to_str(char* str, size_t len, struct BytesBuffer buf)
{
    if (len < buf.len + 1)
        return false;

    memcpy(str, buf.data, len);
    str[len] = 0;

    return true;
}

bool buf_write_token(struct BytesBuffer* buf, char* input,
    struct TokenArray tokens, size_t tokenIndex)
{
    if (tokenIndex >= tokens.len) {
        return false;
    }
    bool ok = true;
    switch (tokens.types[tokenIndex]) {
    case TOK_NUM:
        ok = buf_write_str(buf, "num ", 4);
        break;
    case TOK_STR:
        ok = buf_write_str(buf, "str ", 4);
        break;
    case TOK_SYM:
        ok = buf_write_str(buf, "sym ", 4);
        break;
    case TOK_ERR:
        ok = buf_write_str(buf, "err ", 4);
        break;
    case TOK_END:
        ok = buf_write_str(buf, "end ", 4);
        break;
    }
    buf_write_str_range(buf, input, tokens.starts[tokenIndex],
        tokens.ends[tokenIndex]);
    return ok;
}

enum StackCellType {
    CELL_TYPE_NUM,
    CELL_TYPE_STR,
};

enum StackStatus {
    STACK_OK,
    STACK_OVERFLOW,
    STACK_UNDERFLOW,
};

enum KnownSymbol {
    SYM_ADD,
    SYM_SUB,
    SYM_MUL,
    SYM_DIV,
};

struct StackCell {
    enum StackCellType type;
    union {
        double num;
        char* str;
    };
};

struct StackMachine {
    struct StackCell* stack;
    size_t cap;
    size_t sp;
};

void stack_machine_init(struct StackMachine* machine, struct StackCell* stack, size_t cap)
{
    machine->stack = stack;
    machine->cap = cap;
    machine->sp = 0;
    memset(machine->stack, 0, cap);
}

enum StackStatus stack_machine_push(struct StackMachine* machine, struct StackCell cell)
{
    if (machine->sp >= machine->cap - 1) {
        return STACK_OVERFLOW;
    } else {
        machine->sp++;
        machine->stack[machine->sp] = cell;
    }
    return STACK_OK;
}

enum StackStatus stack_machine_pop(struct StackMachine* machine)
{
    if (machine->sp <= 0) {
        return STACK_UNDERFLOW;
    } else {
        machine->sp--;
    }
    return STACK_OK;
}

struct StackCell stack_machine_peek(struct StackMachine* machine)
{
    if (machine->sp < 0) {
        struct StackCell cell;
        cell.type = CELL_TYPE_NUM;
        cell.num = 0;
        return cell;
    }
    return machine->stack[machine->sp];
}

enum StackStatus stack_machine_exec_sym(struct StackMachine* machine, enum KnownSymbol sym)
{
    enum StackStatus status = STACK_OK;
    struct StackCell op1, op2;
    struct StackCell res;

    switch (sym) {
    case SYM_ADD:
        op1 = stack_machine_peek(machine);
        stack_machine_pop(machine);
        op2 = stack_machine_peek(machine);
        status = stack_machine_pop(machine);
        if (status == STACK_OK && op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            res.type = CELL_TYPE_NUM;
            res.num = op1.num + op2.num;
            stack_machine_push(machine, res);
        }
        break;
    case SYM_SUB:
        op1 = stack_machine_peek(machine);
        stack_machine_pop(machine);
        op2 = stack_machine_peek(machine);
        status = stack_machine_pop(machine);
        if (status == STACK_OK && op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            res.type = CELL_TYPE_NUM;
            res.num = op1.num - op2.num;
            stack_machine_push(machine, res);
        }
        break;
    case SYM_MUL:
        op1 = stack_machine_peek(machine);
        stack_machine_pop(machine);
        op2 = stack_machine_peek(machine);
        status = stack_machine_pop(machine);
        if (status == STACK_OK && op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            res.type = CELL_TYPE_NUM;
            res.num = op1.num * op2.num;
            stack_machine_push(machine, res);
        }
        break;
    case SYM_DIV:
        op1 = stack_machine_peek(machine);
        stack_machine_pop(machine);
        op2 = stack_machine_peek(machine);
        status = stack_machine_pop(machine);
        if (status == STACK_OK && op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            res.type = CELL_TYPE_NUM;
            res.num = op1.num / op2.num;
            stack_machine_push(machine, res);
        }
        break;
    }
    return status;
}

enum StackStatus stack_machine_eval(struct StackMachine *machine, char *input, struct TokenArray tokens)
{
    enum StackStatus status = STACK_OK;
    struct StackCell cell;
    enum KnownSymbol sym;
    char *tokend;
    for (size_t i = 0; i < tokens.len; i++) {
        switch (tokens.types[i]) {
        case TOK_NUM:
            cell.type = CELL_TYPE_NUM;
            cell.num = strtod(&input[tokens.starts[i]], &tokend);
            status = stack_machine_push(machine, cell);
            break;
        case TOK_SYM:
            if (input[tokens.starts[i]] == '+') {
                sym = SYM_ADD;
            } else if (input[tokens.starts[i]] == '-') {
                sym = SYM_SUB;
            } else if (input[tokens.starts[i]] == '*') {
                sym = SYM_MUL;
            } else if (input[tokens.starts[i]] == '/') {
                sym = SYM_DIV;
            }
            status = stack_machine_exec_sym(machine, sym);
            break;
        }
    }
    return status;
}

int main(int argc, char* argv[])
{
    if (argc < 2) {
        return 0;
    } else {
        char* input = argv[1];
        struct TokenArray tokens;
        size_t starts[TOK_CAPACITY];
        size_t ends[TOK_CAPACITY];
        enum TokenType types[TOK_CAPACITY];
        memset(starts, 0, TOK_CAPACITY * sizeof(size_t));
        memset(ends, 0, TOK_CAPACITY * sizeof(size_t));
        memset(types, 0, TOK_CAPACITY * sizeof(size_t));
        tokens.len = 0;
        tokens.cap = TOK_CAPACITY;
        tokens.starts = starts;
        tokens.ends = ends;
        tokens.types = types;

        tokenize_str(&tokens, input, strlen(input));
        struct BytesBuffer buf;
        buf.len = 0;
        int8_t data[OUT_BUF_CAPACITY];
        buf.cap = sizeof data;
        memset(data, 0, sizeof data);
        buf.data = data;
        char output[OUT_BUF_CAPACITY + 1];
        memset(output, 0, sizeof output);

        for (size_t i = 0; i < tokens.len; i++) {
            if (i)
                buf_write_str(&buf, ", ", 2);
            buf_write_token(&buf, input, tokens, i);
        }
        buf_write_to_str(output, sizeof output, buf);
        puts(output);

        struct StackMachine machine;
        struct StackCell stack[255];

        stack_machine_init(&machine, stack, 255);
        enum StackStatus status = stack_machine_eval(&machine, input, tokens);

        printf("stack { status: %d, sp: %ld, top: %lf }\n", status, machine.sp, stack_machine_peek(&machine).num);
    }
}
