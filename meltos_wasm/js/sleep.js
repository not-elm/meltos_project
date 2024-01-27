function sleep_ms(ms){
    return new Promise(resolve => {
        setTimeout(() => {resolve()}, ms)
    })
}
module.exports = {
    sleep_ms
}