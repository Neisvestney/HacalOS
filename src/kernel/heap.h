#ifndef HACALOS_HEAP_H
#define HACALOS_HEAP_H

#include <stdint.h>
#include <stddef.h>

struct HeapSegHdr {
    size_t length;
    HeapSegHdr* next;
    HeapSegHdr* last;
    bool free;
    void CombineForward();
    void CombineBackward();
    HeapSegHdr* Split(size_t splitLength);
};

void initializeHeap(void* heapAddress, size_t pageCount);

void* malloc(size_t size);
void free(void* address);

void expandHeap(size_t length);

void printMap();

void* operator new ( size_t count );
void operator delete ( void* ptr );
void operator delete ( void* ptr, size_t size );

#endif //HACALOS_HEAP_H
