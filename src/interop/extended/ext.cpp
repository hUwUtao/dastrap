#include <daScript/daScript.h>
#include "ext.h"

void iTestTheApiFrNotYetUseful() {
    das::Context * context;
    das::SimFunction * fn = context->getFunction(1);
    context->evalWithCatch(fn, nullptr, nullptr);
}