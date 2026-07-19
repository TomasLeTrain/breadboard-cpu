#include "TemplateDefinitions.hpp"
#include "Opcode.hpp"
#include "TemplateGenerators.hpp"
#include "Templates.hpp"
#include <memory>

IstrTemplateType test_template = {
    {pc_cnt, mar_cnt}, {sp_dec, sp_inc}, {}, {}, {},
};

// IR = [PC]
StepTemplateType universal_step_0 = {mem::read, pc::addr, ir::write};

// pc cnt6
StepTemplateType universal_step_1 = pc::cnt;
StepTemplateType nop = empty_instruction;

// start steps for any instruction that loads an imm16
// warn: must perform pc cnt after
IstrTemplateType load_address_procedure = {
    universal_step_0,
    {pc::cnt},
    {mem::read, pc::addr, mar::hi::write}, // first byte has msb
    {pc::cnt},                             // pc cnt
    {mem::read, pc::addr, mar::lo::write}, // second byte has lsb
};

// clang-format off


// todo: writting register to itself should be error or nop?
// reg0 = reg1
 IstrTemplateType mw_template_reg = {
	universal_step_0,
	universal_step_1,
	{mem::read, pc::addr, ir2::write}, // need to load ir2 to figure out reg1
	{reg1_bout, reg0_write, pc::cnt, reset}, // read from reg1 to reg0, pc cnt
};

// reg = imm8
 IstrTemplateType mw_template_imm = {
	universal_step_0,
	universal_step_1,
	{mem::read , pc::addr , reg0_write}, // write the immediate into reg0
	{reset , pc::cnt}, // pc cnt
};

// reg = [mar]
 IstrTemplateType lw_template_mar = {
	universal_step_0,
	{mem::read , mar::addr , reg0_write , pc::cnt , reset}, // read from addr mar into register, pc cnt
};

// reg = [imm16]
 IstrTemplateType lw_template_imm = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	{mem::read , mar::addr , reg0_write , pc::cnt , reset}, // read from addr mar into register, pc cnt
};


// reg = [mar]
 IstrTemplateType sw_template_mar = {
	universal_step_0,
	{mem::write , mar::addr , reg0_bout , pc::cnt , reset}, // read from addr mar into register, pc cnt
};


// reg = [imm16]
 IstrTemplateType sw_template_imm = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	{mem::write , mar::addr , reg0_bout , pc::cnt , reset}, // read from addr mar into register, pc cnt
};

// [sp--] = reg
 IstrTemplateType push_template_reg = {
	universal_step_0,
	{pc::cnt , sp::dec}, // decrement before pushing value
	{mem::write , sp::addr , reg0_bout , reset}, // read from reg into mem at sp addr, pc cnt
};

// [sp--] = imm8, overrides a reg
 IstrTemplateType push_template_imm8 = {
	universal_step_0,
	{pc::cnt , sp::dec}, // decrement before pushing value
	{mem::read , pc::addr , a::write}, // write into ir2
	{mem::write , sp::addr , a::bout , pc::cnt , reset}, // read from ir2 into [sp], pc cnt
};

// reg0 = [sp++]
 IstrTemplateType pop_template = {
	universal_step_0,
	{mem::read , sp::addr , reg0_write , pc::cnt}, // write from [sp] into reg0, pc cnt
	{reset , sp::inc}, // cntrement after popping value
};

// mar = imm16
 IstrTemplateType mar_template_imm16 = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	{reset , pc::cnt}, // pc cnt
};

// jnz reg -> pc = mar if reg != 0 else nop
 IstrTemplateType jnz_template_reg = {
	universal_step_0,
	{a::write , reg0_bout , pc::cnt}, // note: pc cnt happens in case jump doesn't happens
	{flags::alu_write}, // write zero result to flag register
	{mar::hi::bout , pc::hi::write , flags::select::zero},
	{mar::lo::bout , pc::lo::write , flags::select::zero , reset},
};

// can save instruction if a is already loaded
 IstrTemplateType jnz_template_reg_a = {
	universal_step_0,
	{flags::alu_write,pc::cnt}, // update flag register
	{mar::hi::bout , pc::hi::write , flags::select::zero},
	{mar::lo::bout , pc::lo::write , flags::select::zero , reset},
};

// jump if equal flag is carry flag is true
 IstrTemplateType jmp_imm16_template = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	{pc::cnt}, // note: pc cnt happens in case jump doesn't happens
	{mar::hi::bout , pc::hi::write , output_flags_selector}, // load from mar into pc if flag
    {mar::lo::bout , pc::lo::write , output_flags_selector , reset}, // load from mar into pc if flag
};

// jump if equal flag is true
 IstrTemplateType jmp_mar_template = {
	universal_step_0,
	{pc::cnt}, // note: pc cnt in case jump doesn't happen
	{mar::hi::bout , pc::hi::write , output_flags_selector},
    {mar::lo::bout , pc::lo::write , output_flags_selector , reset},
};

// todo: all math variants could have faster variants if reg0/reg1 are equal to a/b
// todo: special case if reg0 = b, reg1 = a (impossible to swap registers without intermediate)

// reg0 = reg0 op reg1
 IstrTemplateType math_carry_template_reg = {
	universal_step_0,
	universal_step_1,
	{mem::read , pc::addr , ir2::write}, // need to load ir2 to figure out reg1
	{reg0_bout , a::write , pc::cnt}, // load reg0 into a
	reg1_bout , b::write, // load reg1 into b
	f::bout , flag_write_alu , reg0_write , pc_flag_carry, // do math op, save to reg0, writes to flag reg
	reset,
};

 IstrTemplateType math_no_carry_template_reg = {
	universal_step_0,
	universal_step_1,
	mem::read , pc::addr , ir2::write, // need to load ir2 to figure out reg1
	reg0_bout , a::write , pc::cnt, // load reg0 into a
	reg1_bout , b::write, // load reg1 into b
	f::bout , flag_write_alu , reg0_write , pc_flag_direct, // do math op, save to reg0, writes to flag reg
	reset,
};

// reg0 = reg0 op reg1
 IstrTemplateType math_carry_template_imm = {
	universal_step_0,
	reg0_bout , a::write , pc::cnt, // load reg0 into a first (in case reg0 = b), pc cnt
	mem::read , pc::addr , b::write, // load imm into b
	f::bout , flag_write_alu , reg0_write , pc_flag_carry , pc::cnt, // save f to reg0, writes to flag reg
	reset,
};

 IstrTemplateType math_no_carry_template_imm = {
	universal_step_0,
	reg0_bout , a::write , pc::cnt, // load reg0 into a first (in case reg0 = b), pc cnt
	mem::read , pc::addr , b::write, // load imm into b
	f::bout , flag_write_alu , reg0_write , pc_flag_direct , pc::cnt, // save f to reg0, writes to flag reg
	reset,
};

// reg0 = ~reg0
 IstrTemplateType not_template_none = {
	universal_step_0,
	reg0_bout , a::write , pc::cnt, // load reg0 into a, pc cnt
	f::bout , flag_write_alu , reg0_write, // do math op, save to reg0, writes to flag reg
	reset,
};

// reg0 = ~reg1
 IstrTemplateType not_template_reg = {
	universal_step_0,
	universal_step_1,
	mem::read , pc::addr , ir2::write, // need to load ir2 to figure out reg1
	reg1_bout , a::write , pc::cnt, // load reg0 into a
	f::bout , flag_write_alu  , reg0_write, // do math op, save to reg1, writes to flag reg
	reset,
};


// sp dec
 IstrTemplateType sp_dec_template = {
	universal_step_0,
	pc::cnt , sp::dec , reset, // decrement sp
};

// sp cnt
 IstrTemplateType sp_cnt_template = {
	universal_step_0,
	pc::cnt , sp::cnt , reset,
};


// mar cnt
 IstrTemplateType mar_cnt_template = {
	universal_step_0,
	pc::cnt , mar::cnt , reset, // cntrement mar
};

// mar <- pc
 IstrTemplateType pc_to_mar_template = {
	universal_step_0,
	universal_step_1,
	pc::hi::bout , mar::hi::write,
	pc::lo::bout , mar::lo::write , reset,
};

// mar <- sp
 IstrTemplateType sp_to_mar_template = {
	universal_step_0,
	sp::hi::bout , mar::hi::write , pc::cnt,
	sp::lo::bout , mar::lo::write , reset,
};

// sp <- mar
 IstrTemplateType mar_to_sp_template = {
	universal_step_0,
	mar::hi::bout , sp::hi::write , pc::cnt,
	mar::lo::bout , sp::lo::write , reset,
};

// sp <- imm16
 IstrTemplateType sp_template_imm16 = {
    universal_step_0,
    pc::cnt,
    mem::read , pc::addr , sp::hi::write, // write first part of address to sp lo
    pc::cnt,                            // pc cnt
    mem::read , pc::addr , sp::lo::write, // write second part of address to sp hi
	reset , pc::cnt,
};


// reg0 = reg0 op reg1
 IstrTemplateType cmp_template_reg = {
	universal_step_0,
	universal_step_1,
	mem::read , pc::addr , ir2::write, // need to load ir2 to figure out reg1
	reg0_bout , a::write , pc::cnt, // load reg0 into a
	reg1_bout , b::write, // load reg1 into b
	flag_write_alu, // writes to flag reg
	reset,
};

// reg0 = reg0 op reg1
 IstrTemplateType cmp_template_imm = {
	universal_step_0,
	reg0_bout , a::write , pc::cnt, // load reg0 into a first (in case reg0 = b), pc cnt
	mem::read , pc::addr , b::write, // load imm into b
	flag_write_alu , pc::cnt,
	reset,
};

// reg0 = keyboard input
 IstrTemplateType keyboard_template = {
	universal_step_0,
	keyb::bout , reg0_write , pc::cnt , reset,
};


// reg0 = keyboard input
 IstrTemplateType update_flag_register_template = {
	universal_step_0,
	flag_write_alu , pc::cnt,
	reset,
};


 IstrTemplateType halt_template = {
	universal_step_0,
	halt,
};

// 2 instruction nop
 IstrTemplateType nop_template = {
	universal_step_0,
	pc::cnt , reset,
};

 IstrTemplateType vram_read_template_no_delay = {
	universal_step_0,
	vram::bout , mar::addr, // note: must add register write manually
	nop,
	pc::cnt , reset,
};

 IstrTemplateType vram_read_template_delay = {
	universal_step_0,
	nop,
	vram::bout , mar::addr, // note: must add register write manually
	pc::cnt , reset,
};


 IstrTemplateType vram_write_template = {
	universal_step_0,
	vram::write , mar::addr,
	vram::write , mar::addr, // note: must add register bout manually
	pc::cnt , reset, /// todo: can add mar::cnt
};








// global instance of the instruction set generated by the function below
static InstructionSet istr_set;

void instantiateTemplates() {
  // TODO: implement all the different templates
  istr_set.instructions[0] = std::make_unique<Instruction>(test_template);
}

IstrTemplateType opcodeToTemplate(const Opcode &opcode) {
  // TODO: implement
  return istr_set.opcodeToTemplate(opcode);
}
