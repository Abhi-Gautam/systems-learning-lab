#include "impl.h"
FreeListAllocator::FreeListAllocator(int b):memory_(nullptr),capacity_(b),first_(nullptr){/* TODO */} FreeListAllocator::~FreeListAllocator(){/* TODO*/}
void*FreeListAllocator::allocate(int){return nullptr;} void FreeListAllocator::deallocate(void*){/* TODO*/} int FreeListAllocator::bytes_free()const{return 0;}
