# TODO List

## High Priority

- [ ] **Password Export Implementation**
  - [ ] Chrome/Edge password decryption (OS-specific keychain access)
  - [ ] Firefox password extraction from key4.db
  - [ ] Safari password extraction from Keychain
  - [ ] Add secure password handling (no plaintext in logs)

## Medium Priority

- [ ] **Enhanced Browser Support**
  - [ ] Brave browser support
  - [ ] Vivaldi browser support
  - [ ] Opera browser support
  - [ ] Edge Canary/Dev/Beta profiles detection

- [ ] **Performance Optimizations**
  - [ ] Parallel export for multiple browsers
  - [ ] Large database handling optimization
  - [ ] Memory usage optimization for large exports

- [ ] **Data Validation**
  - [ ] YAML schema validation
  - [ ] Duplicate detection and removal
  - [ ] Broken URL detection
  - [ ] Export data integrity checks

## Low Priority

- [ ] **Advanced Features**
  - [ ] Export to other formats (JSON, CSV, HTML)
  - [ ] Bookmark synchronization between browsers
  - [ ] Bookmark deduplication across browsers
  - [ ] Bookmark statistics and analytics
  - [ ] CLI progress bars for large exports

- [ ] **User Experience**
  - [ ] Interactive mode with browser selection
  - [ ] Configuration file support
  - [ ] Verbose output modes
  - [ ] Colored output for better readability

- [ ] **Platform Enhancements**
  - [ ] Windows Certificate Store integration
  - [ ] Linux Secret Service integration
  - [ ] Android data directory support
  - [ ] iOS simulator data access

## Technical Debt

- [ ] **Code Quality**
  - [ ] Remove unused code and imports
  - [ ] Add comprehensive unit tests
  - [ ] Add integration tests
  - [ ] Improve error messages clarity

- [ ] **Documentation**
  - [ ] API documentation generation
  - [ ] Code comments and documentation
  - [ ] User guide with screenshots
  - [ ] Troubleshooting guide

## Security Considerations

- [ ] **Security Hardening**
  - [ ] Input validation and sanitization
  - [ ] Safe handling of sensitive data
  - [ ] Audit logging for security events
  - [ ] Permissions and access control

## Known Issues

- [ ] Safari bookmarks require manual copying on macOS
- [ ] Firefox database requires browser to be closed
- [ ] Windows registry access not implemented
- [ ] Large history exports may cause memory issues

## Future Releases

### v0.2.0

- Password export support
- Brave and Vivaldi browser support
- JSON export format

### v0.3.0

- Web-based UI
- Bookmark synchronization
- Advanced filtering and search

### v1.0.0

- Full browser ecosystem support
- Comprehensive security audit
- Production-ready stability
