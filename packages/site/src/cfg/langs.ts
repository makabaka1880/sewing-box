// Created by Sean L. on Jun. 16.
// Last Updated by Sean L. on Jun. 16.
// 
// sewing-box
// src/cfg/langs.ts
// 
// Makabaka1880, 2026. All rights reserved.

export interface LangProfile {
    name: string,
    description: string,
    grammar: Production[],
    sample: string
}

export interface Production {
    nonterminal: string,
    produces: string[]
}

export const profiles: LangProfile[] = [
    {
        name: "Lambda",
        description: "An implementation of the Untyped Lambda Calculus that does reduction up to WHNF (Weak-Head Normal Form) using Krivine's Machine.",
        grammar: [
            {
                nonterminal: "t",
                produces: ["(lam x t)", "(app t t)"],
            },
            {
                nonterminal: "x",
                produces: ["any identifier"]
            }
        ],
        sample: "(app (lam x x) (lam y y))"
    },
    {
        name: "Stacky",
        description: "A stack-based combinator language. Push atoms onto a stack, then plant them as branches of a tree.",
        grammar: [
            {
                nonterminal: "stmt",
                produces: ["(push s)", "(plant s n)"],
            },
            {
                nonterminal: "s",
                produces: ["any string"]
            },
            {
                nonterminal: "n",
                produces: ["any non-negative integer"]
            },
            {
                nonterminal: "program",
                produces: ["(stmt ...)"]
            }
        ],
        sample: `((push bread)
(push lettuce)
(push cheese)
(push ham)
(plant filling 3)
(push bread)
(plant sandwich 3))
        `
    },
    {
        name: "Lite80",
        description: "A small S-expression–based assembly DSL for the Intel 8080. Write symbolic instructions, get a memory image.",
        grammar: [
            {
                nonterminal: "program",
                produces: ["(block addr item ...) ..."],
            },
            {
                nonterminal: "addr",
                produces: ["hex literal", "decimal integer"],
            },
            {
                nonterminal: "item",
                produces: ["instr", "(db byte ...)"],
            },
            {
                nonterminal: "instr",
                produces: ["(mnemonic operand ...)"],
            },
            {
                nonterminal: "mnemonic",
                produces: ["mov", "mvi", "lxi", "lda", "sta", "add", "adi", "sub", "inr", "dcr", "inx", "dcx", "jmp", "jcond", "call", "ret", "push", "pop", "hlt", "nop", "…"],
            },
            {
                nonterminal: "operand",
                produces: ["register", "hex literal", "character literal", "condition code"],
            },
            {
                nonterminal: "register",
                produces: ["A", "B", "C", "D", "E", "H", "L", "M"],
            },
            {
                nonterminal: "byte",
                produces: ["hex byte", "decimal byte", "'c'"],
            },
        ],
        sample: `(block 0100h
  (lxi SP 0F000h)
  (mvi A '>')
  (out 01h)
  (mvi A 20h)
  (out 01h)
  (hlt))

(block 0200h
  (db 'H' 'e' 'l' 'l' 'o' 0Ah 00h))`,
    },
    {
        name: "Brainfuck",
        description: "An esoteric, Turing-complete language with an 8-instruction pointer-based tape machine. Minimal syntax, maximal confusion.",
        grammar: [
            {
                nonterminal: "instr",
                produces: ["+", "-", ">", "<", "[", "]", ".", ","]
            },
            {
                nonterminal: "program",
                produces: ["instr ..."]
            }
        ],
        sample: `>++++++++[<+++++++++>-]<.
>++++[<+++++++>-]<+.
+++++++..
+++.
>>++++++[<+++++++>-]<++.
------------.
>++++++[<+++++++++>-]<+.
<.
+++.
------.
--------.
>>>++++[<++++++++>-]<+.`
    }
]

export function getLang(name: string): LangProfile | undefined {
    return profiles.find(p => p.name.toLowerCase() === name.toLowerCase())
}

export function getGrammar(name: string): Production[] | undefined {
    return getLang(name)?.grammar
}

export function getSample(name: string): string | undefined {
    return getLang(name)?.sample
}

export function getDescription(name: string): string | undefined {
    return getLang(name)?.description
}

export function generateEBNF(grammar: Production[]): string {
    return grammar
        .map(rule => {
            const rhs = rule.produces
                .map(alt => JSON.stringify(alt))
                .join(' | ')
            return `${rule.nonterminal} = ${rhs} ;`
        })
        .join('\n')
}