<?php
  $content = file_get_contents("php://input");
  $json    = json_decode($content, true);
  echo $json['one'] . ' ' . $json['two'];
?>
