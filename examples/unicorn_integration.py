"""
// example.exe

int examplefn() {
    int a = 10;
    int b = 20;
    int c = 30;
    int d = a + b;
    int e = c - d;
    return e + d;
}

int main() {
    int result = examplefn();
    printf("result = %d\n", result);
    
    system("PAUSE");
    return 0;
}
"""
from unicorn import *
from unicorn.x86_const import *
from memflow import *

PAGE_SIZE = 1024 * 4  # 4 kb
FUNCTION_OFFSET = 0x1000

inventory = Inventory()
os = inventory.create_os("native")
process = os.process_from_name("example.exe")
module = process.module_by_name("example.exe")


def addr_page_aligned(addr):
    return addr - addr % PAGE_SIZE


# callback for tracing invalid memory access (READ or WRITE)
def hook_mem_invalid(uc, access, address, size, value, user_data):
    # We will map in memory by page
    page_addr = addr_page_aligned(address)
    print(">>> Mapping in memory page 0x%x" % (page_addr))
    page_bytes = process.read(page_addr, c_byte * PAGE_SIZE)
    uc.mem_map(page_addr, PAGE_SIZE)
    uc.mem_write(page_addr, bytes(page_bytes))
    return True


# callback for tracing basic blocks
def hook_block(uc, address, size, user_data):
    print(">>> Tracing basic block at 0x%x, block size = 0x%x" % (address, size))


# callback for tracing instructions
def hook_code(uc, address, size, user_data):
    print(">>> Tracing instruction at 0x%x, instruction size = 0x%x" % (address, size))


mu = Uc(UC_ARCH_X86, UC_MODE_64)
# Intercept invalid memory events
mu.hook_add(
    UC_HOOK_MEM_READ_UNMAPPED | UC_HOOK_MEM_WRITE_UNMAPPED | UC_HOOK_MEM_FETCH_UNMAPPED,
    hook_mem_invalid,
)

mu.hook_add(UC_HOOK_CODE, hook_code)

# execute till print
start = module.base + FUNCTION_OFFSET

# map initial executable page
mu.mem_map(addr_page_aligned(start), PAGE_SIZE)
page_bytes = process.read(addr_page_aligned(start), c_ubyte * PAGE_SIZE)
mu.mem_write(
    addr_page_aligned(start),
    bytes(page_bytes),
)

mu.emu_start(start, start + 0x49)
result = mu.reg_read(UC_X86_REG_EAX)
print(f">>> result = {result}")
