use ::riscv::register::sstatus;

pub unsafe fn user_access_begin(){
    sstatus::set_sum();
}

pub unsafe fn user_access_end(){
    sstatus::clear_sum();
}
