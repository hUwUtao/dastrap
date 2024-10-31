#include "ext.h"

// static void iTestTheApiFrNotYetUseful() {
//     das::Context * context;
//     das::SimFunction * fn = context->getFunction(1);
//     context->evalWithCatch(fn, nullptr, nullptr);
// }

static bool dasx_verif_fn(das::SimFunction * fun, char * name) {
    if (fun != nullptr && name != nullptr) {
        if (strcmp(fun->name, name)) {
            return true;
        }
    }
    return false;
}