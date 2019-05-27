for ext in dmg exe AppImage
do
    mc cp build/*-$(cat build/.version).$ext pixelpoint/kpf/kube-forwarder-$(cat build/.version)-$(cat build/.number).$ext
done
