import gdb

class LinkedListAllocatorPrettyPrinter:
    def __init__(self, val):
        self.val = val
    
    def to_string(self):
        result = "Allocator: "
        block = self.val['free_list']

        # result += "Sentinel" #f"[{block['size']}: {block['next']}]"

        block = block['next']

        # result += str(block.type.fields())

        # result += str(block["Some"]["__0"].dereference())

        while self.is_some(block):
            address = block["Some"]["__0"]
            block = address.dereference()

            result += f" -> {address}({block["size"]})"

            block = block['next']

        return result
    
    def is_some(self, option):
        try:
            option["Some"]
        except Exception:
            return False
        else:
            return True

    
def lookup_pretty_printer(val):
    if str(val.type) == "graph_os::allocator::ll_alloc::LinkedListAllocator":
        return LinkedListAllocatorPrettyPrinter(val)
    else:
        return None

gdb.pretty_printers.append(lookup_pretty_printer)