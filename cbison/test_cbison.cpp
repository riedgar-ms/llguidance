// test_cbison.cpp
#include <cassert>
#include <iostream>
#include <vector>
#include <algorithm>
#include <utility>
#include "cbison.hpp"
#include "llguidance_cbison.h"

int main() {
  cbison::Tokenizer t(llg_new_cbison_byte_tokenizer());
  cbison::Factory f(llg_new_cbison_factory_json(t.get(), "{}", nullptr, 0));

  // validate grammar
  {
    auto [ok, msg] = f.validateGrammar("json", "{}");
    assert(ok && msg.empty());
  }
  {
    auto [ok, msg] = f.validateGrammar("json", "foobar");
    assert(!ok);
    assert(msg.find("expected ident") != std::string::npos);
  }

  // error on bad grammar
  {
    auto m_err = f.newMatcher("json", "foobar");
    auto err = m_err.getError();
    assert(err && err->find("expected ident") != std::string::npos);
  }

  // matcher on valid grammar
  auto m = f.newMatcher("json", "{}");
  assert(!m.getError());
  assert(!m.isAccepting());

  // validate_tokens for incomplete JSON
  auto tokens = t.tokenizeString("{\"a\":abc}");
  int n_valid = m.validateTokens(tokens);
  assert(n_valid < static_cast<int>(tokens.size()));

  // validate & consume for complete JSON
  tokens = t.tokenizeString("{\"a\":12}");
  n_valid = m.validateTokens(tokens);
  assert(n_valid == static_cast<int>(tokens.size()));
  assert(!m.isAccepting());
  m.consumeTokens(tokens);
  assert(m.isAccepting());
  assert(m.isStopped());

  // rollback and clone
  m.rollback(3);
  auto m2 = m.clone();
  assert(!m.isAccepting());
  assert(!m.isStopped());

  // consume last 3 tokens
  std::vector<uint32_t> last3(tokens.end() - 3, tokens.end());
  m.consumeTokens(last3);
  assert(m.isAccepting());
  assert(m.isStopped());

  // reset and re-consume full stream
  m.reset();
  assert(!m.isAccepting());
  assert(!m.isStopped());
  m.consumeTokens(tokens);
  assert(m.isAccepting());
  assert(m.isStopped());

  // m2 independent state
  assert(!m2.isAccepting());
  assert(!m2.isStopped());
  m2.consumeTokens(last3);
  assert(m2.isAccepting());
  assert(m2.isStopped());

  // compute mask and ff tokens
  m2.rollback(1);
  auto mask2 = m2.computeMask();
  for (auto v : mask2)
    std::cout << v << ' ';
  std::cout << '\n';
  auto ff = m2.computeFFTokens();
  assert(ff.empty());

  // batch compute masks
  m.rollback(1);
  size_t batch = 3;
  size_t words = f.maskByteLen() / 4;
  std::vector<uint32_t> mask(batch * words, 0);
  std::vector<std::pair<cbison::Matcher *, uint32_t *>> reqs = {
      {&m, mask.data()}, {&m2, mask.data() + 2 * words}};
  int rc = f.computeMasks(reqs);
  assert(rc == 0);

  // verify rows
  std::vector<uint32_t> row0(mask.begin(), mask.begin() + words);
  std::vector<uint32_t> row2(mask.begin() + 2 * words,
                             mask.begin() + 3 * words);
  assert(row0 == mask2);
  assert(row2 == mask2);
  for (size_t i = words; i < 2 * words; ++i)
    assert(mask[i] == 0);

  std::cout << "All tests passed\n";
  return 0;
}