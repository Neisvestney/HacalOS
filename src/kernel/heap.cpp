#include "heap.h"
#include "paging/PageFrameAllocator.h"
#include "paging/PageTableManager.h"
#include "BasicRenderer.h"

void* heapStart;
void* heapEnd;
HeapSegHdr* lastHdr;

void initializeHeap(void* heapAddress, size_t pageCount){
    void* pos = heapAddress;

    for (size_t i = 0; i < pageCount; i++){
        kernelPageTableManager.MapMemory(pos, globalPageFrameAllocator.RequestPage());
        pos = (void*)((size_t)pos + 0x1000);
    }

    size_t heapLength = pageCount * 0x1000;

    heapStart = heapAddress;
    heapEnd = (void*)((size_t)heapStart + heapLength);
    HeapSegHdr* startSeg = (HeapSegHdr*)heapAddress;
    startSeg->length = heapLength - sizeof(HeapSegHdr);
    startSeg->next = nullptr;
    startSeg->last = nullptr;
    startSeg->free = true;
    lastHdr = startSeg;
}

void free(void* address){
    HeapSegHdr* segment = (HeapSegHdr*)address - 1;
    segment->free = true;
    segment->CombineForward();
    segment->CombineBackward();
}

void* malloc(size_t size){
    if (size % 0x10 > 0){ // it is not a multiple of 0x10
        size -= (size % 0x10);
        size += 0x10;
    }

    if (size == 0) return NULL;

    HeapSegHdr* currentSeg = (HeapSegHdr*) heapStart;
    while(true){
        if(currentSeg->free){
            if (currentSeg->length > size){
                currentSeg->Split(size);
                currentSeg->free = false;
                return (void*)((uint64_t)currentSeg + sizeof(HeapSegHdr));
            }
            if (currentSeg->length == size){
                currentSeg->free = false;
                return (void*)((uint64_t)currentSeg + sizeof(HeapSegHdr));
            }
        }
        if (currentSeg->next == NULL) break;
        currentSeg = currentSeg->next;
    }
    expandHeap(size);
    return malloc(size);
}

HeapSegHdr* HeapSegHdr::Split(size_t splitLength){
    if (splitLength < 0x10) return NULL;
    int64_t splitSegLength = length - splitLength - (sizeof(HeapSegHdr));
    if (splitSegLength < 0x10) return NULL;

    HeapSegHdr* newSplitHdr = (HeapSegHdr*) ((size_t)this + splitLength + sizeof(HeapSegHdr));
    next->last = newSplitHdr; // Set the next segment's last segment to our new segment
    newSplitHdr->next = next; // Set the new segment's next segment to out original next segment
    next = newSplitHdr; // Set our new segment to the new segment
    newSplitHdr->last = this; // Set our new segment's last segment to the current segment
    newSplitHdr->length = splitSegLength; // Set the new header's length to the calculated value
    newSplitHdr->free = free; // make sure the new segment's free is the same as the original
    length = splitLength; // set the length of the original segment to its new length

    if (lastHdr == this) lastHdr = newSplitHdr;
    return newSplitHdr;
}

void expandHeap(size_t length){
    if (length % 0x1000) {
        length -= length % 0x1000;
        length += 0x1000;
    }

    size_t pageCount = length / 0x1000;
    HeapSegHdr* newSegment = (HeapSegHdr*)heapEnd;

    for (size_t i = 0; i < pageCount; i++){
        kernelPageTableManager.MapMemory(heapEnd, globalPageFrameAllocator.RequestPage());
        heapEnd = (void*)((size_t)heapEnd + 0x1000);
    }

    newSegment->free = true;
    newSegment->last = lastHdr;
    lastHdr->next = newSegment;
    lastHdr = newSegment;
    newSegment->next = NULL;
    newSegment->length = length - sizeof(HeapSegHdr);
    newSegment->CombineBackward();

}

void HeapSegHdr::CombineForward(){
    if (next == NULL) return;
    if (!next->free) return;

    if (next == lastHdr) lastHdr = this;

    if (next->next != NULL){
        next->next->last = this;
    }

    length = length + next->length + sizeof(HeapSegHdr);
}

void printMap() {
    HeapSegHdr* seg = (HeapSegHdr*)heapStart;
    while (true) {
        basicRenderer.Print("@");
        for (int i = 0; i < seg->length; i += 0x100) {
            if (seg->free) basicRenderer.PutChar('.');
            else basicRenderer.PutChar('#');
        }
        if (seg->next == nullptr) break;
        seg = seg->next;
    }
    basicRenderer.NextLine();
}

void HeapSegHdr::CombineBackward(){
    if (last != NULL && last->free) last->CombineForward();
}

void* operator new ( size_t count ) {
    return malloc(count);
}

void operator delete ( void* ptr ) {
    free(ptr);
}

void operator delete ( void* ptr, size_t size ) {
    free(ptr);
}