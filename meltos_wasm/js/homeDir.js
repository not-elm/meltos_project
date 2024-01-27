function home_dir() { return process.env[process.platform == 'win32' ? 'USERPROFILE' : 'HOME']; }
module.exports = {
    home_dir
}