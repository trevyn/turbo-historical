go get golang.org/dl/go1.15.5
go1.15.5 download

- adjust scrapertest.yml
- adjust build-turbo.sh
- adjust appcenter-post-clone.sh
- adjust rclone/package.json

updating rclone fork:

git rebase upstream/master
(resolve any conflicts)
git push --force

see:
https://github.com/rclone/rclone/issues/4685
https://github.com/rclone/rclone/issues/3257
