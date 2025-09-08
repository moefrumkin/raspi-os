pub struct BuddyAllocator<'a> {
    heap: &'a [u8],
    free_list: FreeList
}

struct FreeList {

}