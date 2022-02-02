#!/bin/bash
# assuming a PR is checked out we can test for regressions from the merge base
# in bazel using the following logic.

set -e
set -x

SUMMARY_DIR=bazel-summary ;

fetchPR () {
  git fetch origin -f pull/"$1"/head:$2 1>/dev/null
  git checkout "$2" 1>/dev/null
}

summarizeRef () {
  mkdir -p $SUMMARY_DIR/results

  COMMIT=$(git rev-parse $1)
  STATUS_FILE="$SUMMARY_DIR/results/$COMMIT.log"

  if [ ! -f "$STATUS_FILE" ]; then
    git checkout $COMMIT 1>/dev/null
    git clean -n 1>/dev/null
    # Run a big bazel test on the whole repo. This is a nice thing if you're going
    # to maintain a cache.
    bazel test //... --keep_going --test_tag_filters=-cw310,-failing_verilator \
      --disk_cache=/usr/local/google/home/drewmacrae/bazel_cache \
      --local_test_jobs=HOST_CPUS*0.2 --test_summary=short \
      --test_output=summary \
      | grep '//' \
      | sed 's/ (cached) / / ; s/ in / / ; s/ [0-9]\+.[0-9]\+s// ; s/ \+/ /g' \
      | sort > "$STATUS_FILE"
  fi

  echo "$STATUS_FILE"
}

vetPR () {
  git fetch
  git switch master
  git pull

  fetchPR $1 pr$1

  PR_HEAD=$(git rev-parse HEAD)
  MERGE_BASE=$(git merge-base HEAD master)

  # Don't stop if you identify a diff
  { diff $(summarizeRef $MERGE_BASE) $(summarizeRef $PR_HEAD) > $SUMMARY_DIR/pr$1status.diff ; diffreturn="$?" ; } || true
  case $diffreturn in
    0)
      echo "pr$1 looks good";
      echo $'\a' ;;
    1)
      wall "I found a diff in pr$1's bazel results storing in $SUMMARY_DIR/pr$1status.diff" ;
      echo "Compared merge base: $MERGE_BASE to pr head: $PR_HEAD" >> $SUMMARY_DIR/pr$1status.diff ;;
    *)
      echo "diff returned $diffreturn which indicates trouble with pr$1" ;
      false ;;
  esac
}

for PR in $(gh pr list -L 200 --state open --search "draft:false" | awk '{print $1}') ; do
  vetPR $PR
done
