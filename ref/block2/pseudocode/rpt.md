RPT — C-like pseudocode (Interrupt Program)

/* Transfer control to interrupting program and save context. */
void RPT(void) {
    // Save Z and B to ZRUPT and BRUPT
    ZRUPT = Z;
    BRUPT = B;
    // Set Z to address provided by Interrupt Priority Control
    Z = InterruptPriorityControl.get_interrupt_address();
    // Inhibit further interrupts
    inhibit_interrupts();
    // Reset priority request
    InterruptPriorityControl.reset_request();
    // Execute STD2 to start interrupt handler
    SQG.execute_STD2();
}
