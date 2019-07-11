address1=48432b505b46b73dc8bf66d215639c08d86b96a447320ea2ed358893f3cc7016
msginit1=48432b50-msg-init.boc
msgbody1=48432b50-msg-body.boc
address2=94532a2c9f228586431c6997390948075872b70992abc1f9c0aa3a1256a32fbd
msginit2=94532a2c-msg-init.boc

rm -f *.tvc *.boc *.tmp

source set_env.sh

$linker --lib ../stdlib_sol.tvm ./contract03-a.code --abi-json ./contract03-a.abi.json
$linker --lib ../stdlib_sol.tvm ./contract03-b.code --abi-json ./contract03-b.abi.json

if [ ! -f "${address1}.tvc" ]; then
  echo "FILE NOT FOUND! ${address1}.tvc"
  exit 1
fi

if [ ! -f "${address2}.tvc" ]; then
  echo "FILE NOT FOUND! ${address2}.tvc"
  exit 1
fi

$linker $address1 message --init -w 0
$linker $address2 message --init -w 0

$linker $address1 message -w 0 --abi-json contract03-a.abi.json --abi-method method_external \
	--abi-params "{\"anotherContract\":\"0x${address2}\"}"

zeroes=0000000000000000000000000000000000000000000000000000000000000000

$emulator/create-msg --type transfer --src $zeroes --dst $address1 --value 1000000 --out sendmoney1.boc
$emulator/create-msg --type transfer --src $zeroes --dst $address2 --value 1000000 --out sendmoney2.boc


echo "-------------------------------"
$tlc -f sendmoney1.boc
$tlc -f sendmoney2.boc
echo "-------------------------------"
sleep 5
echo "-------------------------------"
$tlc -a 0:$address1
$tlc -a 0:$address2

echo "-------------------------------"
$tlc -f $msginit1
$tlc -f $msginit2
echo "-------------------------------"
sleep 5

echo "-------------------------------"
$tlc -a 0:$address1
$tlc -a 0:$address2

echo "-------------------------------"
$tlc -f $msgbody1
echo "-------------------------------"
sleep 5
echo "-------------------------------"
$tlc -a 0:$address1
sleep 5
echo "-------------------------------"
$tlc -a 0:$address2