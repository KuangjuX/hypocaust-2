# Trap 的设计

## 由 hypervisor 切换到 guest
- 切换 `hgatp` 寄存器并且 `hfence.gvma` 用来开启两阶段页表翻译
- 设置 `hsatus` 寄存器的 `SPV` 位为 1 表明返回 VS mode
- 设置 `sstatus` 寄存器的 `SPP` 位为 1 表明返回 S mode
- 设置 `sepc` 用来标志返回入口

## 由 guest 切换到 hypervisor
- 首先交换 `sscratch` 和 `sp` 寄存器的值,此时 `sscratch` 存储的是 guest 栈指针地址,`sp` 存储的是 trap context 地址
- 保存上下文到 trap context 内存区域
- 将 `sp` 寄存器切换到内核栈并跳转到 trap 处理函数(在 H 扩展情况下不需要切换页表)
  
由于此时在 guest 和 hypervisor 之间不再需要切换页表了,有一些事情需要注意:
- hypervisor 需要单独映射 Trap Context Page,而 guest 则不需要进行映射
- guest 内核栈的虚拟地址与 Trap Context page 的地址不能重合,否则会破坏 Trap Context 的结构