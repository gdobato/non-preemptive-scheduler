
/* 
 * Linker script extension for STM32F429I
 */

MEMORY
{
  FLASH(rx): ORIGIN = 0x08000000, LENGTH = 2048K
  RAM(rxw) : ORIGIN = 0x20000000, LENGTH = 192K
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM); 
