#include "keyboardScancodeTranslation.h"

namespace QWERTYKeyboard {
    const char asciiTable[] = {
            0 ,  0 , '1', '2',
            '3', '4', '5', '6',
            '7', '8', '9', '0',
            '-', '=',  0 ,  0 ,
            'q', 'w', 'e', 'r',
            't', 'y', 'u', 'i',
            'o', 'p', '[', ']',
            0 ,  0 , 'a', 's',
            'd', 'f', 'g', 'h',
            'j', 'k', 'l', ';',
            '\'','`',  0 , '\\',
            'z', 'x', 'c', 'v',
            'b', 'n', 'm', ',',
            '.', '/',  0 , '*',
            0 , ' '
    };

    const char shiftAsciiTable[] = {
            0 ,  0 , '!', '@',
            '#', '$', '%', '^',
            '&', '*', '(', ')',
            '_', '+',  0 ,  0 ,
            'q', 'w', 'e', 'r',
            't', 'y', 'u', 'i',
            'o', 'p', '{', '}',
            0 ,  0 , 'a', 's',
            'd', 'f', 'g', 'h',
            'j', 'k', 'l', ':',
            '"','~',  0 , '|',
            'z', 'x', 'c', 'v',
            'b', 'n', 'm', '<',
            '>', '?',  0 , '*',
            0 , ' '
    };

    char Translate(uint8_t scancode, bool uppercase, bool shiftPressed) {
        if (scancode > 58) return 0;

        if (uppercase && ((scancode >= 16 && scancode <= 25) || (scancode >= 30 && scancode <= 38) || (scancode >= 44 && scancode <= 50))){
            return (char)(asciiTable[scancode] - 32);
        }
        else if (shiftPressed) return shiftAsciiTable[scancode];
        else return asciiTable[scancode];
    }
}